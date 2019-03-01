package texlab

import com.google.gson.JsonPrimitive
import kotlinx.coroutines.*
import kotlinx.coroutines.future.await
import kotlinx.coroutines.future.future
import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import texlab.build.BuildConfig
import texlab.build.BuildEngine
import texlab.build.BuildParams
import texlab.build.BuildResult
import texlab.completion.*
import texlab.completion.bibtex.BibtexCitationGenerator
import texlab.completion.bibtex.BibtexEntryTypeProvider
import texlab.completion.bibtex.BibtexFieldNameProvider
import texlab.completion.bibtex.BibtexKernelCommandProvider
import texlab.completion.latex.*
import texlab.completion.latex.data.LatexComponentDatabase
import texlab.completion.latex.data.symbols.LatexArgumentSymbolProvider
import texlab.completion.latex.data.symbols.LatexCommandSymbolProvider
import texlab.completion.latex.data.symbols.LatexSymbolDatabase
import texlab.definition.*
import texlab.diagnostics.*
import texlab.folding.*
import texlab.formatting.BibtexFormatter
import texlab.formatting.BibtexFormatterConfig
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
import texlab.resolver.InvalidTexDistributionException
import texlab.resolver.LatexResolver
import texlab.resolver.TexDistributionError
import texlab.search.ForwardSearchConfig
import texlab.search.ForwardSearchResult
import texlab.search.ForwardSearchTool
import texlab.symbol.*
import texlab.syntax.LatexSyntaxTree
import texlab.syntax.bibtex.BibtexDeclarationSyntax
import java.io.File
import java.net.URI
import java.nio.file.Path
import java.nio.file.Paths
import java.util.concurrent.CompletableFuture
import kotlin.coroutines.CoroutineContext

@ObsoleteCoroutinesApi
class TextDocumentServiceImpl(val workspaceActor: WorkspaceActor) : CustomTextDocumentService, CoroutineScope {
    override val coroutineContext: CoroutineContext = Dispatchers.Default + SupervisorJob()

    private lateinit var client: CustomLanguageClient

    private val progressListener = object : ProgressListener {
        override fun onReportProgress(params: ProgressParams) {
            client.progress(params)
        }
    }

    private val resolver: Deferred<LatexResolver> = async(start = CoroutineStart.LAZY) {
        try {
            LatexResolver.create()
        } catch (e: InvalidTexDistributionException) {
            val message = when (e.error) {
                TexDistributionError.KPSEWHICH_NOT_FOUND ->
                    """An error occured while executing `kpsewhich`.
                        |Please make sure that your distribution is in your PATH environment variable
                        |and provides the `kpsewhich` tool.
                    """.trimMargin()
                TexDistributionError.UNKNOWN_DISTRIBUTION ->
                    """Your TeX distribution is not supported.
                        |Please install a supported distribution.
                    """.trimMargin()
                TexDistributionError.INVALID_DISTRIBUTION ->
                    """Your installed TeX distribution seems to be corrupt.
                        |Please reinstall your distribution.
                    """.trimMargin()
            }

            client.showMessage(MessageParams(MessageType.Error, message))
            LatexResolver.empty()
        }
    }

    private val serverDirectory: Path = Paths.get(javaClass.protectionDomain.codeSource.location.toURI()).parent

    private val componentDatabase: Deferred<LatexComponentDatabase> = async(start = CoroutineStart.LAZY) {
        val databaseFile = serverDirectory.resolve("components.json").toFile()
        LatexComponentDatabase.loadOrCreate(databaseFile, resolver.await(), progressListener)
    }

    private val symbolDatabase: Deferred<LatexSymbolDatabase> = async {
        val databaseDirectory = serverDirectory.resolve("symbols")
        LatexSymbolDatabase.loadOrCreate(databaseDirectory)
    }

    private val includeGraphicsProvider: IncludeGraphicsProvider = IncludeGraphicsProvider()
    private val completionProvider: CompletionProvider =
            LimitedCompletionProvider(
                    OrderByQualityProvider(
                            AggregateCompletionProvider(
                                    includeGraphicsProvider,
                                    LatexIncludeProvider(),
                                    LatexInputProvider(),
                                    LatexBibliographyProvider(),
                                    DeferredCompletionProvider(::LatexClassImportProvider, resolver),
                                    DeferredCompletionProvider(::LatexPackageImportProvider, resolver),
                                    PgfLibraryProvider,
                                    TikzLibraryProvider,
                                    LatexCitationProvider,
                                    LatexColorProvider,
                                    DefineColorModelProvider,
                                    DefineColorSetModelProvider,
                                    LatexLabelProvider,
                                    LatexBeginCommandProvider,
                                    DeferredCompletionProvider(::LatexComponentEnvironmentProvider, componentDatabase),
                                    LatexKernelEnvironmentProvider,
                                    LatexUserEnvironmentProvider,
                                    DeferredCompletionProvider(::LatexArgumentSymbolProvider, symbolDatabase),
                                    DeferredCompletionProvider(::LatexCommandSymbolProvider, symbolDatabase),
                                    DeferredCompletionProvider(::TikzCommandProvider, componentDatabase),
                                    DeferredCompletionProvider(::LatexComponentCommandProvider, componentDatabase),
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

    fun connect(client: CustomLanguageClient) {
        this.client = client

        launch {
            while (true) {
                workspaceActor.withWorkspace { workspace ->
                    workspace.documents.map { workspace.relatedDocuments(it.uri) }
                            .forEach { componentDatabase.await().getRelatedComponents(it) }
                }

                delay(1000)
            }
        }
    }

    fun initialize(root: Path?) {
        includeGraphicsProvider.root = root
    }

    override fun didOpen(params: DidOpenTextDocumentParams) {
        launch {
            params.textDocument.apply {
                val language = getLanguageById(languageId) ?: return@launch
                val uri = URIHelper.parse(uri)
                workspaceActor.put { Document.create(uri, text, language) }
                publishDiagnostics(uri)
            }
        }
    }

    override fun didChange(params: DidChangeTextDocumentParams) {
        assert(params.contentChanges.size == 1)
        launch {
            val uri = URIHelper.parse(params.textDocument.uri)
            workspaceActor.put { workspace ->
                val oldDocument = workspace.documents.first { it.uri == uri }
                val text = params.contentChanges[0].text
                oldDocument.copy(text, LatexSyntaxTree(text))
            }

            publishDiagnostics(uri)
        }
    }

    override fun didSave(params: DidSaveTextDocumentParams) {
        launch {
            val uri = URIHelper.parse(params.textDocument.uri)
            val config = client.configuration<BuildConfig>("latex.build", uri)
            if (config.onSave) {
                workspaceActor.withWorkspace { workspace ->
                    val document = workspace.findParent(uri)
                    val identifier = TextDocumentIdentifier(document.uri.toString())
                    build(BuildParams(identifier)).await()
                }
            }
        }
    }

    override fun didClose(params: DidCloseTextDocumentParams) {
    }

    override fun documentSymbol(params: DocumentSymbolParams)
            : CompletableFuture<MutableList<Either<SymbolInformation, DocumentSymbol>>?> = future {
        workspaceActor.withWorkspace { workspace ->
            val uri = URIHelper.parse(params.textDocument.uri)
            val document = workspace.documents
                    .firstOrNull { it.uri == uri }
                    ?: return@withWorkspace null

            val request = SymbolRequest(document)
            symbolProvider.getSymbols(request)
                    .map { Either.forRight<SymbolInformation, DocumentSymbol>(it) }
                    .toMutableList()
        }
    }

    override fun rename(params: RenameParams): CompletableFuture<WorkspaceEdit?> = future {
        workspaceActor.withWorkspace { workspace ->
            val uri = URIHelper.parse(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = RenameRequest(uri, relatedDocuments, params.position, params.newName)
            renamer.rename(request)
        }
    }

    override fun documentLink(params: DocumentLinkParams)
            : CompletableFuture<MutableList<DocumentLink>> = future {
        workspaceActor.withWorkspace { workspace ->
            val uri = URIHelper.parse(params.textDocument.uri)
            val request = LinkRequest(workspace, uri)
            linkProvider.getLinks(request).toMutableList()
        }
    }

    override fun completion(params: CompletionParams)
            : CompletableFuture<Either<MutableList<CompletionItem>, CompletionList>> = future {
        workspaceActor.withWorkspace { workspace ->
            val uri = URIHelper.parse(params.textDocument.uri)
            val request = CompletionRequest(uri, params.position, workspace)
            val items = completionProvider.complete(request).toList()
            val list = CompletionList(true, items)
            Either.forRight<MutableList<CompletionItem>, CompletionList>(list)
        }
    }

    override fun resolveCompletionItem(unresolved: CompletionItem)
            : CompletableFuture<CompletionItem> = future {
        if (unresolved.kind == CompletionItemKind.Constant) {
            val entry = unresolved.data as JsonPrimitive

            unresolved.setDocumentation(MarkupContent().apply {
                kind = MarkupKind.MARKDOWN
                value = BibtexCitationGenerator.cite(entry.asString)
            })

            return@future unresolved
        }

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
        workspaceActor.withWorkspace { workspace ->
            val uri = URIHelper.parse(params.textDocument.uri)
            val document = workspace.documents
                    .firstOrNull { it.uri == uri }
                    ?: return@withWorkspace null

            val request = FoldingRequest(document)
            foldingProvider.fold(request).toMutableList()
        }
    }

    override fun definition(params: TextDocumentPositionParams)
            : CompletableFuture<MutableList<out Location>?> = future {
        workspaceActor.withWorkspace { workspace ->
            val uri = URIHelper.parse(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = DefinitionRequest(uri, relatedDocuments, params.position)
            val location = definitionProvider.find(request)
            location?.let { mutableListOf(it) }
        }
    }

    override fun hover(params: TextDocumentPositionParams): CompletableFuture<Hover?> = future {
        workspaceActor.withWorkspace { workspace ->
            val uri = URIHelper.parse(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = HoverRequest(uri, relatedDocuments, params.position)
            hoverProvider.getHover(request)
        }
    }

    override fun formatting(params: DocumentFormattingParams)
            : CompletableFuture<MutableList<out TextEdit>?> = future {
        val uri = URIHelper.parse(params.textDocument.uri)
        val config = client.configuration<BibtexFormatterConfig>("bibtex.formatting", uri)
        workspaceActor.withWorkspace { workspace ->
            val document = workspace.documents
                    .filterIsInstance<BibtexDocument>()
                    .firstOrNull { it.uri == uri }
                    ?: return@withWorkspace null
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
        workspaceActor.withWorkspace { workspace ->
            val uri = URIHelper.parse(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = ReferenceRequest(uri, relatedDocuments, params.position)
            referenceProvider.getReferences(request)?.toMutableList()
        }
    }

    override fun documentHighlight(params: TextDocumentPositionParams)
            : CompletableFuture<MutableList<out DocumentHighlight>?> = future {
        workspaceActor.withWorkspace { workspace ->
            val uri = URIHelper.parse(params.textDocument.uri)
            val document = workspace.documents.firstOrNull { it.uri == uri }
                    ?: return@withWorkspace null

            val request = HighlightRequest(document, params.position)
            highlightProvider.getHighlights(request)?.toMutableList()
        }
    }

    override fun build(params: BuildParams): CompletableFuture<BuildResult> = future {
        workspaceActor.withWorkspace { workspace ->
            val childUri = URIHelper.parse(params.textDocument.uri)
            val parent = workspace.findParent(childUri)
            val config = client.configuration<BuildConfig>("latex.build", parent.uri)
            BuildEngine.build(parent.uri, config, progressListener)
        }
    }

    override fun forwardSearch(params: TextDocumentPositionParams)
            : CompletableFuture<ForwardSearchResult> = future {
        workspaceActor.withWorkspace { workspace ->
            val childUri = URIHelper.parse(params.textDocument.uri)
            val parent = workspace.findParent(childUri)
            val config = client.configuration<ForwardSearchConfig>("latex.forwardSearch", parent.uri)
            ForwardSearchTool.search(File(childUri), File(parent.uri), params.position.line, config)

        }
    }

    suspend fun publishDiagnostics(uri: URI) {
        workspaceActor.withWorkspace { workspace ->
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = DiagnosticsRequest(uri, relatedDocuments)
            val diagnostics = diagnosticsProvider.getDiagnostics(request)
            val params = PublishDiagnosticsParams(uri.toString(), diagnostics)
            client.publishDiagnostics(params)
        }
    }
}
