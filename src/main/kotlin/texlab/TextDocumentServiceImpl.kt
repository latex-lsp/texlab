package texlab

import com.google.gson.JsonPrimitive
import kotlinx.coroutines.*
import kotlinx.coroutines.future.await
import kotlinx.coroutines.future.future
import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import texlab.build.BuildConfig
import texlab.build.BuildEngine
import texlab.build.BuildParams
import texlab.build.BuildResult
import texlab.completion.OrderByQualityProvider
import texlab.completion.bibtex.BibtexCitationActor
import texlab.completion.bibtex.BibtexEntryTypeProvider
import texlab.completion.bibtex.BibtexFieldNameProvider
import texlab.completion.bibtex.BibtexKernelCommandProvider
import texlab.completion.latex.*
import texlab.completion.latex.data.LatexComponentDatabase
import texlab.completion.latex.data.symbols.LatexArgumentSymbolProvider
import texlab.completion.latex.data.symbols.LatexCommandSymbolProvider
import texlab.completion.latex.data.symbols.LatexSymbolDatabase
import texlab.definition.BibtexEntryDefinitionProvider
import texlab.definition.LatexLabelDefinitionProvider
import texlab.diagnostics.BibtexEntryDiagnosticsProvider
import texlab.diagnostics.ManualDiagnosticsProvider
import texlab.folding.BibtexDeclarationFoldingProvider
import texlab.folding.LatexEnvironmentFoldingProvider
import texlab.folding.LatexSectionFoldingProvider
import texlab.formatting.BibtexFormatter
import texlab.formatting.BibtexFormatterConfig
import texlab.highlight.LatexLabelHighlightProvider
import texlab.hover.*
import texlab.link.LatexIncludeLinkProvider
import texlab.metadata.BibtexEntryTypeMetadataProvider
import texlab.metadata.LatexComponentMetadataProvider
import texlab.provider.*
import texlab.references.BibtexEntryReferenceProvider
import texlab.references.LatexLabelReferenceProvider
import texlab.rename.BibtexEntryRenamer
import texlab.rename.LatexCommandRenamer
import texlab.rename.LatexEnvironmentRenamer
import texlab.rename.LatexLabelRenamer
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
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths
import java.util.concurrent.CompletableFuture
import kotlin.coroutines.CoroutineContext

@ObsoleteCoroutinesApi
class TextDocumentServiceImpl(val workspaceActor: WorkspaceActor) : CustomTextDocumentService, CoroutineScope {
    companion object {
        private val logger: Logger = LoggerFactory.getLogger("")
    }

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

    private val homeDirectory: Path = Paths.get(System.getProperty("user.home"))
    private val databaseDirectory: Path = homeDirectory.resolve(".texlab")

    init {
        if (!Files.exists(databaseDirectory)) {
            Files.createDirectory(databaseDirectory)
        }
    }

    private val componentDatabase: Deferred<LatexComponentDatabase> = async(start = CoroutineStart.LAZY) {
        val databaseFile = databaseDirectory.resolve("components.json").toFile()
        LatexComponentDatabase.loadOrCreate(databaseFile, resolver.await(), progressListener)
    }

    private val symbolDatabase: Deferred<LatexSymbolDatabase> = async {
        val databaseDirectory = serverDirectory.resolve("symbols")
        LatexSymbolDatabase.loadOrCreate(databaseDirectory)
    }

    private val includeGraphicsProvider: IncludeGraphicsProvider = IncludeGraphicsProvider()

    private val completionProvider: FeatureProvider<CompletionParams, CompletionItem> =
            LimitedProvider(
                    OrderByQualityProvider(
                            DistinctProvider(
                                    AggregateProvider(
                                            includeGraphicsProvider,
                                            LatexIncludeProvider(),
                                            LatexInputProvider(),
                                            LatexBibliographyProvider(),
                                            DeferredProvider(::LatexClassImportProvider, resolver),
                                            DeferredProvider(::LatexPackageImportProvider, resolver),
                                            PgfLibraryProvider,
                                            TikzLibraryProvider,
                                            LatexCitationProvider,
                                            LatexColorProvider,
                                            DefineColorModelProvider,
                                            DefineColorSetModelProvider,
                                            LatexLabelProvider,
                                            LatexBeginCommandProvider,
                                            DeferredProvider(::LatexComponentEnvironmentProvider, componentDatabase),
                                            LatexKernelEnvironmentProvider,
                                            LatexUserEnvironmentProvider,
                                            DeferredProvider(::LatexArgumentSymbolProvider, symbolDatabase),
                                            DeferredProvider(::LatexCommandSymbolProvider, symbolDatabase),
                                            DeferredProvider(::TikzCommandProvider, componentDatabase),
                                            DeferredProvider(::LatexComponentCommandProvider, componentDatabase),
                                            LatexKernelCommandProvider,
                                            LatexUserCommandProvider,
                                            BibtexEntryTypeProvider,
                                            BibtexFieldNameProvider,
                                            BibtexKernelCommandProvider)) { it.label }))

    private val symbolProvider: FeatureProvider<DocumentSymbolParams, DocumentSymbol> =
            AggregateProvider(
                    LatexCommandSymbolProvider,
                    LatexEnvironmentSymbolProvider,
                    LatexLabelSymbolProvider,
                    LatexCitationSymbolProvider,
                    BibtexEntrySymbolProvider)

    private val renameProvider: FeatureProvider<RenameParams, WorkspaceEdit> =
            AggregateProvider(
                    LatexCommandRenamer,
                    LatexEnvironmentRenamer,
                    LatexLabelRenamer,
                    BibtexEntryRenamer)

    private val foldingProvider: FeatureProvider<FoldingRangeRequestParams, FoldingRange> =
            AggregateProvider(
                    LatexEnvironmentFoldingProvider,
                    LatexSectionFoldingProvider,
                    BibtexDeclarationFoldingProvider)

    private val linkProvider: FeatureProvider<DocumentLinkParams, DocumentLink> =
            AggregateProvider(LatexIncludeLinkProvider)

    private val definitionProvider: FeatureProvider<TextDocumentPositionParams, Location> =
            AggregateProvider(
                    LatexLabelDefinitionProvider,
                    BibtexEntryDefinitionProvider)

    private val highlightProvider: FeatureProvider<TextDocumentPositionParams, DocumentHighlight> =
            AggregateProvider(LatexLabelHighlightProvider)

    private val hoverProvider: FeatureProvider<TextDocumentPositionParams, Hover> =
            AggregateProvider(
                    LatexComponentHoverProvider,
                    LatexCitationHoverProvider,
                    LatexMathEnvironmentHoverProvider,
                    LatexMathEquationHoverProvider,
                    LatexMathInlineHoverProvider,
                    DeferredProvider(::LatexCommandHoverProvider, componentDatabase),
                    BibtexEntryTypeHoverProvider,
                    BibtexFieldHoverProvider)

    private val referenceProvider: FeatureProvider<ReferenceParams, Location> =
            AggregateProvider(
                    LatexLabelReferenceProvider,
                    BibtexEntryReferenceProvider)

    val buildDiagnosticsProvider: ManualDiagnosticsProvider = ManualDiagnosticsProvider()

    private val diagnosticsProvider: FeatureProvider<Unit, Diagnostic> =
            AggregateProvider(
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
        params.textDocument.apply {
            val language = getLanguageById(languageId) ?: return
            val uri = URIHelper.parse(uri)
            workspaceActor.put { Document.create(uri, text, language) }

            launch {
                publishDiagnostics(uri)
            }
        }
    }

    override fun didChange(params: DidChangeTextDocumentParams) {
        assert(params.contentChanges.size == 1)
        val uri = URIHelper.parse(params.textDocument.uri)
        workspaceActor.put { workspace ->
            val oldDocument = workspace.documents.first { it.uri == uri }
            val text = params.contentChanges[0].text
            oldDocument.copy(text, LatexSyntaxTree(text))
        }

        launch {
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
            : CompletableFuture<List<Either<SymbolInformation, DocumentSymbol>>> = future {
        runFeature(symbolProvider, params.textDocument, params)
                .map { Either.forRight<SymbolInformation, DocumentSymbol>(it) }
    }

    override fun rename(params: RenameParams): CompletableFuture<WorkspaceEdit?> = future {
        runFeature(renameProvider, params.textDocument, params).firstOrNull()
    }

    override fun documentLink(params: DocumentLinkParams)
            : CompletableFuture<List<DocumentLink>> = future {
        runFeature(linkProvider, params.textDocument, params)
    }

    override fun completion(params: CompletionParams)
            : CompletableFuture<Either<List<CompletionItem>, CompletionList>> = future {
        val items = runFeature(completionProvider, params.textDocument, params)
        val list = CompletionList(true, items)
        Either.forRight<List<CompletionItem>, CompletionList>(list)
    }

    override fun resolveCompletionItem(unresolved: CompletionItem)
            : CompletableFuture<CompletionItem> = future {
        if (unresolved.kind == CompletionItemKind.Constant) {
            val entry = unresolved.data as JsonPrimitive
            val citation = BibtexCitationActor.cite(entry.asString)

            unresolved.setDocumentation(MarkupContent().apply {
                kind = MarkupKind.MARKDOWN
                value = citation
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
            : CompletableFuture<List<FoldingRange>> = future {
        runFeature(foldingProvider, params.textDocument, params)
    }

    override fun definition(params: TextDocumentPositionParams)
            : CompletableFuture<List<Location>> = future {
        runFeature(definitionProvider, params.textDocument, params)
    }

    override fun hover(params: TextDocumentPositionParams)
            : CompletableFuture<Hover?> = future {
        runFeature(hoverProvider, params.textDocument, params).firstOrNull()
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
            : CompletableFuture<List<Location>> = future {
        runFeature(referenceProvider, params.textDocument, params)
    }

    override fun documentHighlight(params: TextDocumentPositionParams)
            : CompletableFuture<List<DocumentHighlight>> = future {
        runFeature(highlightProvider, params.textDocument, params)
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
            val request = FeatureRequest(uri, workspace, Unit, logger)
            val diagnostics = diagnosticsProvider.get(request)
            val params = PublishDiagnosticsParams(uri.toString(), diagnostics)
            client.publishDiagnostics(params)
        }
    }

    private suspend fun <T, R> runFeature(provider: FeatureProvider<T, R>,
                                          document: TextDocumentIdentifier,
                                          params: T): List<R> {
        return workspaceActor.withWorkspace { workspace ->
            val uri = URIHelper.parse(document.uri)
            val request = FeatureRequest(uri, workspace, params, logger)
            provider.get(request)
        }
    }
}
