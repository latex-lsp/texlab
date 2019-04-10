package texlab

import com.google.gson.JsonPrimitive
import kotlinx.coroutines.*
import kotlinx.coroutines.future.await
import kotlinx.coroutines.future.future
import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.eclipse.lsp4j.jsonrpc.services.JsonDelegate
import org.eclipse.lsp4j.services.LanguageServer
import org.eclipse.lsp4j.services.WorkspaceService
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import texlab.build.*
import texlab.completion.CompletionProvider
import texlab.completion.bibtex.BibtexCitationActor
import texlab.completion.latex.data.LatexComponentDatabase
import texlab.definition.DefinitionProvider
import texlab.diagnostics.DiagnosticsProvider
import texlab.diagnostics.LatexLinterConfig
import texlab.folding.FoldingProvider
import texlab.formatting.BibtexFormatter
import texlab.formatting.BibtexFormatterConfig
import texlab.highlight.LatexLabelHighlightProvider
import texlab.hover.HoverProvider
import texlab.link.LatexIncludeLinkProvider
import texlab.metadata.BibtexEntryTypeMetadataProvider
import texlab.metadata.LatexComponentMetadataProvider
import texlab.metadata.MetadataProvider
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.references.ReferenceProvider
import texlab.rename.RenameProvider
import texlab.resolver.InvalidTexDistributionException
import texlab.resolver.LatexResolver
import texlab.resolver.TexDistributionError
import texlab.search.ForwardSearchConfig
import texlab.search.ForwardSearchResult
import texlab.search.ForwardSearchTool
import texlab.symbol.SymbolProvider
import texlab.syntax.BibtexSyntaxTree
import texlab.syntax.LatexSyntaxTree
import texlab.syntax.bibtex.BibtexDeclarationSyntax
import java.io.File
import java.io.IOException
import java.net.URI
import java.nio.file.FileSystems
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths
import java.util.concurrent.CompletableFuture
import kotlin.coroutines.CoroutineContext
import kotlin.streams.toList

@ObsoleteCoroutinesApi
class LatexLanguageServer : LanguageServer, LatexTextDocumentService, WorkspaceService, CoroutineScope {
    override val coroutineContext: CoroutineContext = Dispatchers.Default + SupervisorJob()

    private val logger: Logger = LoggerFactory.getLogger("")
    private lateinit var client: LatexLanguageClient

    private var workspaceRootDirectory: Path? = null
    private val workspaceActor = WorkspaceActor()

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
                    """An error occurred while executing `kpsewhich`.
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

    private val componentDatabase: Deferred<LatexComponentDatabase> = async(start = CoroutineStart.LAZY) {
        LatexComponentDatabase.loadOrCreate(
                LatexLanguageServerConfig.COMPONENT_DATABASE_FILE.toFile(),
                resolver.await(),
                progressListener)
    }

    private val completionProvider = CompletionProvider(resolver, componentDatabase)
    private val hoverProvider = HoverProvider(componentDatabase)
    private val diagnosticsProvider = DiagnosticsProvider()

    @JsonDelegate
    override fun getTextDocumentService(): LatexTextDocumentService = this

    override fun getWorkspaceService(): WorkspaceService = this

    fun connect(client: LatexLanguageClient) {
        this.client = client

        launch {
            while (true) {
                workspaceActor.withWorkspace { workspace ->
                    workspace.documentsByUri.values
                            .map { workspace.relatedDocuments(it.uri) }
                            .forEach { componentDatabase.await().getRelatedComponents(it) }
                }

                delay(1000)
            }
        }
    }

    override fun initialize(params: InitializeParams): CompletableFuture<InitializeResult> = future {
        if (params.rootUri != null && params.rootUri.startsWith("file")) {
            val root = URIHelper.parse(params.rootUri)
            loadWorkspace(root)
            workspaceRootDirectory = Paths.get(root)
        }

        val capabilities = ServerCapabilities().apply {
            val syncOptions = TextDocumentSyncOptions().apply {
                openClose = true
                save = SaveOptions(true)
                change = TextDocumentSyncKind.Full
            }
            textDocumentSync = Either.forRight(syncOptions)
            documentSymbolProvider = true
            renameProvider = Either.forLeft(true)
            documentLinkProvider = DocumentLinkOptions(false)
            completionProvider = CompletionOptions(true, listOf("\\", "{", "}", "@", "/"))
            foldingRangeProvider = Either.forLeft(true)
            definitionProvider = true
            hoverProvider = true
            documentFormattingProvider = true
            referencesProvider = true
            documentHighlightProvider = true
        }

        launch {
            resolveIncludes()
        }

        InitializeResult(capabilities)
    }

    override fun initialized(params: InitializedParams?) {
        val watcher = FileSystemWatcher("**/*.log", WatchKind.Create or WatchKind.Change)
        val options = DidChangeWatchedFilesRegistrationOptions(listOf(watcher))
        val registration = Registration("log-watcher", "workspace/didChangeWatchedFiles", options)
        client.registerCapability(RegistrationParams(listOf(registration)))
    }

    override fun didChangeWatchedFiles(params: DidChangeWatchedFilesParams) {
        launch {
            for (change in params.changes) {
                val logPath = File(URIHelper.parse(change.uri)).toPath()
                val texPath = logPath.resolveSibling(logPath.toFile().nameWithoutExtension + ".tex")
                val texUri = texPath.toUri()

                workspaceActor.withWorkspace { workspace ->
                    val document = workspace.documentsByUri[texUri]
                    if (document != null) {
                        try {
                            val log = withContext(Dispatchers.IO) {
                                Files.readAllBytes(logPath).toString(Charsets.UTF_8)
                            }
                            val allErrors = BuildErrorParser.parse(texUri, log)

                            diagnosticsProvider.buildProvider.diagnosticsByUri = allErrors
                                    .groupBy { it.uri }
                                    .mapValues { errors -> errors.value.map { it.toDiagnostic() } }

                            workspace.documentsByUri.values.forEach { publishDiagnostics(it.uri) }
                        } catch (e: IOException) {
                            // File is still locked
                        }
                    }
                }
            }
        }
    }

    override fun didChangeConfiguration(params: DidChangeConfigurationParams) {
    }

    override fun didOpen(params: DidOpenTextDocumentParams) {
        params.textDocument.apply {
            val language = getLanguageById(languageId) ?: return
            val uri = URIHelper.parse(uri)
            workspaceActor.put { Document.create(uri, text, language) }

            launch {
                runLinter(uri, text)
                publishDiagnostics(uri)
                resolveIncludes()
            }
        }
    }

    override fun didChange(params: DidChangeTextDocumentParams) {
        assert(params.contentChanges.size == 1)
        val uri = URIHelper.parse(params.textDocument.uri)
        workspaceActor.put { workspace ->
            val oldDocument = workspace.documentsByUri.getValue(uri)
            val text = params.contentChanges[0].text
            val tree = when (oldDocument) {
                is LatexDocument -> LatexSyntaxTree(text)
                is BibtexDocument -> BibtexSyntaxTree(text)
            }
            oldDocument.copy(text, tree)
        }

        launch {
            publishDiagnostics(uri)
            resolveIncludes()
        }
    }

    override fun didSave(params: DidSaveTextDocumentParams) {
        launch {
            val uri = URIHelper.parse(params.textDocument.uri)
            runLinter(uri, params.text)
            publishDiagnostics(uri)

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
        runFeature(SymbolProvider, params.textDocument, params)
                .map { Either.forRight<SymbolInformation, DocumentSymbol>(it) }
    }

    override fun rename(params: RenameParams): CompletableFuture<WorkspaceEdit?> = future {
        runFeature(RenameProvider, params.textDocument, params)
    }

    override fun documentLink(params: DocumentLinkParams)
            : CompletableFuture<List<DocumentLink>> = future {
        runFeature(LatexIncludeLinkProvider, params.textDocument, params)
    }

    override fun completion(params: CompletionParams)
            : CompletableFuture<Either<List<CompletionItem>, CompletionList>> = future {
        val items = runFeature(completionProvider, params.textDocument, params)

        val allIncludes = items.all {
            it.kind == CompletionItemKind.Folder || it.kind == CompletionItemKind.File
        }
        val isIncomplete = !allIncludes || items.size > LatexLanguageServerConfig.COMPLETION_LIMIT
        val list = CompletionList(isIncomplete, items)
        Either.forRight<List<CompletionItem>, CompletionList>(list)
    }

    override fun resolveCompletionItem(unresolved: CompletionItem)
            : CompletableFuture<CompletionItem> = future {
        suspend fun resolveFromMetadataProvider(provider: MetadataProvider) {
            val metadata = provider.getMetadata(unresolved.label)
            unresolved.detail = metadata?.detail
            unresolved.setDocumentation(metadata?.documentation)
        }

        when (unresolved.kind) {
            CompletionItemKind.Constant -> {
                val entry = unresolved.data as JsonPrimitive
                val citation = BibtexCitationActor.cite(entry.asString)

                unresolved.setDocumentation(MarkupContent().apply {
                    kind = MarkupKind.MARKDOWN
                    value = citation
                })
            }
            CompletionItemKind.Class -> {
                resolveFromMetadataProvider(LatexComponentMetadataProvider)
            }
            CompletionItemKind.Interface -> {
                resolveFromMetadataProvider(BibtexEntryTypeMetadataProvider)
            }
            else -> {
            }
        }

        unresolved
    }

    override fun foldingRange(params: FoldingRangeRequestParams)
            : CompletableFuture<List<FoldingRange>> = future {
        runFeature(FoldingProvider, params.textDocument, params)
    }

    override fun definition(params: TextDocumentPositionParams)
            : CompletableFuture<Either<List<Location>, List<LocationLink>>> = future {
        runFeature(DefinitionProvider, params.textDocument, params)
                .let { Either.forLeft<List<Location>, List<LocationLink>>(it) }
    }

    override fun hover(params: TextDocumentPositionParams)
            : CompletableFuture<Hover?> = future {
        runFeature(hoverProvider, params.textDocument, params)
    }

    override fun formatting(params: DocumentFormattingParams)
            : CompletableFuture<List<TextEdit>?> = future {
        val uri = URIHelper.parse(params.textDocument.uri)
        val config = client.configuration<BibtexFormatterConfig>("bibtex.formatting", uri)
        workspaceActor.withWorkspace { workspace ->
            val document = workspace.documentsByUri[uri] as? BibtexDocument
                    ?: return@withWorkspace null
            val formatter =
                    BibtexFormatter(params.options.isInsertSpaces, params.options.tabSize, config.lineLength)
            document.tree.root.children
                    .filterIsInstance<BibtexDeclarationSyntax>()
                    .map { TextEdit(it.range, formatter.format(it)) }
        }
    }

    override fun references(params: ReferenceParams)
            : CompletableFuture<List<Location>> = future {
        runFeature(ReferenceProvider, params.textDocument, params)
    }

    override fun documentHighlight(params: TextDocumentPositionParams)
            : CompletableFuture<List<DocumentHighlight>> = future {
        runFeature(LatexLabelHighlightProvider, params.textDocument, params)
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

    override fun shutdown(): CompletableFuture<Any?> = future {
        null
    }

    override fun exit() {
    }

    private suspend fun publishDiagnostics(uri: URI) {
        workspaceActor.withWorkspace { workspace ->
            val request = FeatureRequest(uri, workspace, Unit, logger)
            val diagnostics = diagnosticsProvider.get(request)
            val params = PublishDiagnosticsParams(uri.toString(), diagnostics)
            client.publishDiagnostics(params)
        }
    }

    private suspend fun resolveIncludes() {
        workspaceActor.withWorkspace { workspace ->
            for (parent in workspace.documentsByUri.values.filterIsInstance<LatexDocument>()) {
                for (include in parent.tree.includes) {
                    if (workspace.resolveDocument(parent.uri, include.path) != null) {
                        continue
                    }

                    for (target in workspace.resolveLinkTargets(parent.uri, include.path)) {
                        val path = Paths.get(target)
                        val child = Workspace.load(path)
                        if (child != null) {
                            workspaceActor.put { child }
                            break
                        }
                    }
                }
            }
        }
    }

    private suspend fun runLinter(uri: URI, text: String) {
        val config = client.configuration<LatexLinterConfig>("latex.lint", uri)
        if (config.onSave) {
            diagnosticsProvider.latexProvider.update(uri, text)
        } else {
            diagnosticsProvider.latexProvider.clear(uri)
        }
    }

    private suspend fun <T, R> runFeature(provider: FeatureProvider<T, R>,
                                          document: TextDocumentIdentifier,
                                          params: T): R {
        return workspaceActor.withWorkspace { workspace ->
            val uri = URIHelper.parse(document.uri)
            val request = FeatureRequest(uri, workspace, params, logger)
            provider.get(request)
        }
    }

    private suspend fun loadWorkspace(root: URI) {
        if (root.scheme == "file") {
            val matcher = FileSystems.getDefault().getPathMatcher("glob:**.{tex,sty,cls,bib}")

            val files = withContext(Dispatchers.IO) {
                Files.walk(Paths.get(root))
                        .filter { Files.isRegularFile(it) }
                        .filter { matcher.matches(it) }
                        .toList()
            }

            files.forEach { loadWorkspaceFile(it) }
        }
    }

    private suspend fun loadWorkspaceFile(file: Path) {
        val document = Workspace.load(file)
        if (document != null) {
            workspaceActor.put { document }
        }
    }
}
