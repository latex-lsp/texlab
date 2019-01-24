package texlab

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.future.future
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.withLock
import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import texlab.build.BuildConfig
import texlab.build.BuildEngine
import texlab.build.BuildParams
import texlab.build.BuildStatus
import texlab.completion.*
import texlab.completion.bibtex.BibtexEntryTypeProvider
import texlab.completion.bibtex.BibtexFieldNameProvider
import texlab.completion.bibtex.BibtexKernelCommandProvider
import texlab.completion.latex.*
import texlab.completion.latex.data.LatexComponentDatabase
import texlab.completion.latex.data.LatexComponentSourcePrefetcher
import texlab.completion.latex.data.LatexResolver
import texlab.definition.*
import texlab.diagnostics.*
import texlab.folding.*
import texlab.formatting.BibtexFormatter
import texlab.formatting.BibtexFormatterConfig
import texlab.forwardSearch.ForwardSearchConfig
import texlab.forwardSearch.ForwardSearchStatus
import texlab.forwardSearch.ForwardSearchTool
import texlab.highlight.AggregateHighlightProvider
import texlab.highlight.HighlightProvider
import texlab.highlight.HighlightRequest
import texlab.highlight.LatexLabelHighlightProvider
import texlab.hover.*
import texlab.link.AggregateLinkProvider
import texlab.link.LatexIncludeLinkProvider
import texlab.link.LinkProvider
import texlab.link.LinkRequest
import texlab.metadata.BibtexEntryTypeMetadataProvider
import texlab.metadata.LatexComponentMetadataProvider
import texlab.references.*
import texlab.rename.*
import texlab.symbol.*
import texlab.syntax.bibtex.BibtexDeclarationSyntax
import java.io.File
import java.net.URI
import java.nio.file.Path
import java.nio.file.Paths
import java.util.concurrent.CompletableFuture
import kotlin.coroutines.CoroutineContext

class TextDocumentServiceImpl(override val coroutineContext: CoroutineContext,
                              val workspace: Workspace) : CustomTextDocumentService, CoroutineScope {
    lateinit var client: CustomLanguageClient

    private val progressListener = object : ProgressListener {
        override fun onReportProgress(params: ProgressParams) {
            client.progress(params)
        }
    }

    private val resolver = LatexResolver.create()
    private val databaseDirectory = Paths.get(javaClass.protectionDomain.codeSource.location.toURI()).parent
    private val databaseFile = databaseDirectory.resolve("components.json")
    private val database = LatexComponentDatabase.loadOrCreate(databaseFile, resolver, progressListener)

    init {
        LatexComponentSourcePrefetcher.start(workspace, database)
    }

    private val includeGraphicsProvider: IncludeGraphicsProvider = IncludeGraphicsProvider()
    private val completionProvider: CompletionProvider =
            LimitedCompletionProvider(
                    OrderByQualityProvider(
                            AggregateCompletionProvider(
                                    includeGraphicsProvider,
                                    LatexIncludeProvider(workspace),
                                    LatexInputProvider(workspace),
                                    LatexBibliographyProvider(workspace),
                                    LatexClassImportProvider(resolver),
                                    LatexPackageImportProvider(resolver),
                                    PgfLibraryProvider,
                                    TikzLibraryProvider,
                                    LatexCitationProvider,
                                    LatexColorProvider,
                                    DefineColorModelProvider,
                                    DefineColorSetModelProvider,
                                    LatexLabelProvider,
                                    LatexBeginCommandProvider,
                                    LatexComponentEnvironmentProvider(database),
                                    LatexKernelEnvironmentProvider,
                                    LatexUserEnvironmentProvider,
                                    LatexComponentCommandProvider(database),
                                    LatexKernelCommandProvider,
                                    LatexUserCommandProvider,
                                    BibtexEntryTypeProvider,
                                    BibtexFieldNameProvider,
                                    BibtexKernelCommandProvider)))

    private val symbolProvider: SymbolProvider =
            AggregateSymbolProvider(
                    LatexCommandSymbolProvider,
                    LatexEnvironmentSymbolProvider,
                    LatexLabelSymbolProvider,
                    LatexCitationSymbolProvider,
                    BibtexEntrySymbolProvider)

    private val renamer: Renamer =
            AggregateRenamer(
                    LatexCommandRenamer,
                    LatexEnvironmentRenamer,
                    LatexLabelRenamer,
                    BibtexEntryRenamer)

    private val foldingProvider: FoldingProvider =
            AggregateFoldingProvider(
                    LatexEnvironmentFoldingProvider,
                    LatexSectionFoldingProvider,
                    BibtexDeclarationFoldingProvider)

    private val linkProvider: LinkProvider = AggregateLinkProvider(LatexIncludeLinkProvider)

    private val definitionProvider: DefinitionProvider =
            AggregateDefinitionProvider(
                    LatexLabelDefinitionProvider,
                    BibtexEntryDefinitionProvider)

    private val highlightProvider: HighlightProvider =
            AggregateHighlightProvider(LatexLabelHighlightProvider)

    private val hoverProvider: HoverProvider =
            AggregateHoverProvider(
                    LatexComponentHoverProvider,
                    BibtexEntryTypeHoverProvider,
                    BibtexFieldHoverProvider)

    private val referenceProvider: ReferenceProvider =
            AggregateReferenceProvider(
                    LatexLabelReferenceProvider,
                    BibtexEntryReferenceProvider)

    val buildDiagnosticsProvider: ManualDiagnosticsProvider = ManualDiagnosticsProvider()

    private val diagnosticsProvider: DiagnosticsProvider =
            AggregateDiagnosticsProvider(
                    buildDiagnosticsProvider,
                    BibtexEntryDiagnosticsProvider)

    fun initialize(root: Path?) {
        includeGraphicsProvider.root = root
    }

    override fun didOpen(params: DidOpenTextDocumentParams) {
        launch {
            params.textDocument.apply {
                val language = getLanguageById(languageId) ?: return@launch
                workspace.withLock {
                    val uri = URIHelper.parse(uri)
                    val document = workspace.documents.firstOrNull { it.uri == uri }
                            ?: Document.create(uri, language)
                    if (!workspace.documents.contains(document)) {
                        workspace.documents.add(document)
                    }

                    document.text = text
                    document.analyze()
                    publishDiagnostics(uri)
                }
            }
        }
    }

    override fun didChange(params: DidChangeTextDocumentParams) {
        launch {
            val uri = URIHelper.parse(params.textDocument.uri)
            workspace.withLock {
                val document = workspace.documents.first { it.uri == uri }
                params.contentChanges.forEach { document.text = it.text }
                document.analyze()
                publishDiagnostics(uri)
            }
        }
    }

    override fun didSave(params: DidSaveTextDocumentParams) {
        launch {
            val uri = URIHelper.parse(params.textDocument.uri)
            val config = client.configuration<BuildConfig>("latex.build", uri)
            if (config.onSave) {
                val document = workspace.findParent(uri)
                val identifier = TextDocumentIdentifier(document.uri.toString())
                build(BuildParams(identifier)).get()
            }
        }
    }

    override fun didClose(params: DidCloseTextDocumentParams) {
    }

    override fun documentSymbol(params: DocumentSymbolParams)
            : CompletableFuture<MutableList<Either<SymbolInformation, DocumentSymbol>>?> = future {
        workspace.withLock {
            val uri = URIHelper.parse(params.textDocument.uri)
            val document = workspace.documents
                    .firstOrNull { it.uri == uri }
                    ?: return@future null

            val request = SymbolRequest(document)
            symbolProvider.getSymbols(request)
                    .map { Either.forRight<SymbolInformation, DocumentSymbol>(it) }
                    .toMutableList()
        }
    }

    override fun rename(params: RenameParams): CompletableFuture<WorkspaceEdit?> = future {
        workspace.withLock {
            val uri = URIHelper.parse(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = RenameRequest(uri, relatedDocuments, params.position, params.newName)
            renamer.rename(request)
        }
    }

    override fun documentLink(params: DocumentLinkParams)
            : CompletableFuture<MutableList<DocumentLink>> = future {
        workspace.withLock {
            val uri = URIHelper.parse(params.textDocument.uri)
            val request = LinkRequest(workspace, uri)
            linkProvider.getLinks(request).toMutableList()
        }
    }

    override fun completion(params: CompletionParams)
            : CompletableFuture<Either<MutableList<CompletionItem>, CompletionList>> = future {
        workspace.withLock {
            val uri = URIHelper.parse(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = CompletionRequest(uri, relatedDocuments, params.position)
            val items = completionProvider.complete(request).toList()
            val list = CompletionList(true, items)
            Either.forRight<MutableList<CompletionItem>, CompletionList>(list)
        }
    }

    override fun resolveCompletionItem(unresolved: CompletionItem)
            : CompletableFuture<CompletionItem> = future {
        val provider = when (unresolved.kind) {
            CompletionItemKind.Class -> LatexComponentMetadataProvider
            CompletionItemKind.Interface -> BibtexEntryTypeMetadataProvider
            else -> null
        }

        val metadata = provider?.getMetadata(unresolved.label)
        if (metadata != null) {
            unresolved.detail = metadata.detail
            unresolved.setDocumentation(metadata.documentation)
        }

        unresolved
    }

    override fun foldingRange(params: FoldingRangeRequestParams)
            : CompletableFuture<MutableList<FoldingRange>?> = future {
        workspace.withLock {
            val uri = URIHelper.parse(params.textDocument.uri)
            val document = workspace.documents
                    .firstOrNull { it.uri == uri }
                    ?: return@future null

            val request = FoldingRequest(document)
            foldingProvider.fold(request).toMutableList()
        }
    }

    override fun definition(params: TextDocumentPositionParams)
            : CompletableFuture<MutableList<out Location>?> = future {
        workspace.withLock {
            val uri = URIHelper.parse(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = DefinitionRequest(uri, relatedDocuments, params.position)
            val location = definitionProvider.find(request)
            location?.let { mutableListOf(it) }
        }
    }

    override fun hover(params: TextDocumentPositionParams): CompletableFuture<Hover?> = future {
        val uri = URIHelper.parse(params.textDocument.uri)
        val relatedDocuments = workspace.withLock { workspace.relatedDocuments(uri) }
        val request = HoverRequest(uri, relatedDocuments, params.position)
        hoverProvider.getHover(request)
    }

    override fun formatting(params: DocumentFormattingParams)
            : CompletableFuture<MutableList<out TextEdit>?> = future {
        val uri = URIHelper.parse(params.textDocument.uri)
        val config = client.configuration<BibtexFormatterConfig>("bibtex.formatting", uri)
        workspace.withLock {
            val document =
                    workspace.documents
                            .filterIsInstance<BibtexDocument>()
                            .firstOrNull { it.uri == uri }
                            ?: return@future null
            val formatter =
                    BibtexFormatter(params.options.isInsertSpaces, params.options.tabSize, config.lineLength)
            val edits = mutableListOf<TextEdit>()
            for (entry in document.tree.root.children.filterIsInstance<BibtexDeclarationSyntax>()) {
                edits.add(TextEdit(entry.range, formatter.format(entry)))
            }
            edits
        }
    }

    override fun references(params: ReferenceParams)
            : CompletableFuture<MutableList<out Location>?> = future {
        workspace.withLock {
            val uri = URIHelper.parse(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = ReferenceRequest(uri, relatedDocuments, params.position)
            referenceProvider.getReferences(request)?.toMutableList()
        }
    }

    override fun documentHighlight(params: TextDocumentPositionParams)
            : CompletableFuture<MutableList<out DocumentHighlight>?> = future {
        workspace.withLock {
            val uri = URIHelper.parse(params.textDocument.uri)
            val document = workspace.documents.firstOrNull { it.uri == uri }
                    ?: return@future null

            val request = HighlightRequest(document, params.position)
            highlightProvider.getHighlights(request)?.toMutableList()
        }
    }

    override fun build(params: BuildParams): CompletableFuture<BuildStatus> = future {
        val childUri = URIHelper.parse(params.textDocument.uri)
        val parent = workspace.withLock {
            workspace.findParent(childUri)
        }

        val config = client.configuration<BuildConfig>("latex.build", parent.uri)
        BuildEngine.build(parent.uri, config, progressListener)
    }

    override fun forwardSearch(params: TextDocumentPositionParams)
            : CompletableFuture<ForwardSearchStatus> = future {
        val childUri = URIHelper.parse(params.textDocument.uri)
        val parent = workspace.withLock {
            workspace.findParent(childUri)
        }

        val config = client.configuration<ForwardSearchConfig>("latex.forwardSearch", parent.uri)
        ForwardSearchTool.search(File(childUri), File(parent.uri), params.position.line, config)
    }

    fun publishDiagnostics(uri: URI) {
        val relatedDocuments = workspace.relatedDocuments(uri)
        val request = DiagnosticsRequest(uri, relatedDocuments)
        val diagnostics = diagnosticsProvider.getDiagnostics(request)
        val params = PublishDiagnosticsParams(uri.toString(), diagnostics)
        client.publishDiagnostics(params)
    }
}
