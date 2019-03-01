package texlab

import kotlinx.coroutines.*
import kotlinx.coroutines.future.future
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
import kotlin.streams.toList

@ObsoleteCoroutinesApi
class LanguageServerImpl : LanguageServer, CoroutineScope {
    override val coroutineContext: CoroutineContext = Dispatchers.Default + SupervisorJob()

    private val workspaceActor = WorkspaceActor()
    private val textDocumentService = TextDocumentServiceImpl(workspaceActor)
    private val workspaceService = WorkspaceServiceImpl(textDocumentService)
    private lateinit var client: CustomLanguageClient

    fun connect(client: CustomLanguageClient) {
        textDocumentService.connect(client)
        this.client = client
    }

    override fun initialize(params: InitializeParams): CompletableFuture<InitializeResult> = future {
        if (params.rootUri != null && params.rootUri.startsWith("file")) {
            val root = URIHelper.parse(params.rootUri)
            loadWorkspace(root)
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
        val registration = Registration("log-watcher", "workspaceActor/didChangeWatchedFiles", options)
        client.registerCapability(RegistrationParams(listOf(registration)))
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
        val extension = file.fileName.toFile().extension
        val language = getLanguageByExtension(extension) ?: return
        try {
            workspaceActor.put {
                val text = withContext(Dispatchers.IO) {
                    Files.readAllBytes(file).toString(Charsets.UTF_8)
                }

                Document.create(file.toUri(), text, language)
            }
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
