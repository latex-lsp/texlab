package texlab

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.future.future
import kotlinx.coroutines.sync.withLock
import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.eclipse.lsp4j.jsonrpc.services.JsonDelegate
import org.eclipse.lsp4j.services.LanguageServer
import org.eclipse.lsp4j.services.WorkspaceService
import java.io.IOException
import java.net.URI
import java.nio.file.FileSystems
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths
import java.util.concurrent.CompletableFuture
import kotlin.coroutines.CoroutineContext

class LanguageServerImpl : LanguageServer, CoroutineScope {
    private val workspace: Workspace = Workspace()
    private val textDocumentService = TextDocumentServiceImpl(workspace)
    private val workspaceService = WorkspaceServiceImpl(textDocumentService)
    private lateinit var client: CustomLanguageClient

    override val coroutineContext: CoroutineContext = Dispatchers.Default + SupervisorJob()

    fun connect(client: CustomLanguageClient) {
        textDocumentService.connect(client)
        this.client = client
    }

    override fun initialize(params: InitializeParams): CompletableFuture<InitializeResult> = future {
        if (params.rootUri != null && params.rootUri.startsWith("file")) {
            val root = URIHelper.parse(params.rootUri)
            workspace.withLock {
                loadWorkspace(root)
            }
            textDocumentService.initialize(Paths.get(root))
        } else {
            textDocumentService.initialize(null)
        }

        val capabilities = ServerCapabilities().apply {
            val syncOptions = TextDocumentSyncOptions().apply {
                openClose = true
                save = SaveOptions(false)
                change = TextDocumentSyncKind.Full
            }
            textDocumentSync = Either.forRight(syncOptions)
            documentSymbolProvider = true
            renameProvider = Either.forLeft(true)
            documentLinkProvider = DocumentLinkOptions(false)
            completionProvider = CompletionOptions(true, listOf("\\", "{", "}", "@"))
            foldingRangeProvider = Either.forLeft(true)
            definitionProvider = true
            hoverProvider = true
            documentFormattingProvider = true
            referencesProvider = true
            documentHighlightProvider = true
        }

        InitializeResult(capabilities)
    }


    override fun initialized(params: InitializedParams?) {
        val watcher = FileSystemWatcher("**/*.log", WatchKind.Create or WatchKind.Change)
        val options = DidChangeWatchedFilesRegistrationOptions(listOf(watcher))
        val registration = Registration("log-watcher", "workspace/didChangeWatchedFiles", options)
        client.registerCapability(RegistrationParams(listOf(registration)))
    }

    private fun loadWorkspace(root: URI) {
        if (root.scheme == "file") {
            val matcher = FileSystems.getDefault().getPathMatcher("glob:**.{tex,sty,cls,bib}")
            Files.walk(Paths.get(root))
                    .filter { Files.isRegularFile(it) }
                    .filter { matcher.matches(it) }
                    .forEach { loadWorkspaceFile(it) }
        }
    }

    private fun loadWorkspaceFile(file: Path) {
        val extension = file.fileName.toFile().extension
        val language = getLanguageByExtension(extension) ?: return
        try {
            val text = Files.readAllBytes(file).toString(Charsets.UTF_8)
            val document = Document.create(file.toUri(), text, language)
            workspace.documents.add(document)
        } catch (e: IOException) {
            e.printStackTrace()
        }
    }

    @JsonDelegate
    override fun getTextDocumentService(): CustomTextDocumentService = textDocumentService

    override fun getWorkspaceService(): WorkspaceService = workspaceService

    override fun shutdown(): CompletableFuture<Any?> = future {
        null
    }

    override fun exit() {
    }
}
