package texlab

import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.eclipse.lsp4j.jsonrpc.services.JsonRequest
import org.eclipse.lsp4j.services.LanguageServer
import org.eclipse.lsp4j.services.TextDocumentService
import org.eclipse.lsp4j.services.WorkspaceService
import texlab.build.*
import java.io.IOException
import java.net.URI
import java.nio.file.FileSystems
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths
import java.util.concurrent.CompletableFuture

class LanguageServerImpl : LanguageServer {
    private val workspace: Workspace = Workspace()
    private val textDocumentService = TextDocumentServiceImpl(workspace)
    private val workspaceService = WorkspaceServiceImpl()
    private lateinit var client: LanguageClientExtensions

    fun connect(client: LanguageClientExtensions) {
        this.client = client
        this.textDocumentService.client = client
    }

    override fun initialize(params: InitializeParams): CompletableFuture<InitializeResult> {
        return CompletableFuture.supplyAsync {
            val root = URI.create(params.rootUri)
            synchronized(workspace) {
                loadWorkspace(root)
            }

            val capabilities = ServerCapabilities().apply {
                val syncOptions = TextDocumentSyncOptions().apply {
                    openClose = true
                    change = TextDocumentSyncKind.Full
                }
                textDocumentSync = Either.forRight(syncOptions)
                documentSymbolProvider = true
                renameProvider = Either.forLeft(true)
                documentLinkProvider = DocumentLinkOptions(false)
                completionProvider = CompletionOptions(false, listOf("\\", "{", "}"))
                foldingRangeProvider = Either.forLeft(true)
            }
            InitializeResult(capabilities)
        }
    }

    override fun initialized(params: InitializedParams?) {
        client.setStatus(StatusParams(ServerStatus.IDLE, null))
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
            val document = Document.create(file.toUri(), language)
            document.text = text
            document.analyze()
            workspace.documents.add(document)
        } catch (e: IOException) {
            e.printStackTrace()
        }
    }

    override fun getTextDocumentService(): TextDocumentService = textDocumentService

    override fun getWorkspaceService(): WorkspaceService = workspaceService

    override fun shutdown(): CompletableFuture<Any> {
        return CompletableFuture.completedFuture(null)
    }

    override fun exit() {
    }

    @JsonRequest("textDocument/build", useSegment = false)
    fun build(params: BuildParams): CompletableFuture<BuildStatus> {
        return CompletableFuture.supplyAsync {
            val childUri = URI.create(params.textDocument.uri)
            val parent = workspace.relatedDocuments(childUri)
                    .filterIsInstance<LatexDocument>()
                    .firstOrNull { it.tree.isStandalone }
                    ?: workspace.documents.first { it.uri == childUri }

            val parentName = Paths.get(parent.uri).fileName
            client.setStatus(StatusParams(ServerStatus.BUILDING, parentName.toString()))

            val config = client.configuration<BuildConfig>("latex.build", parent.uri)
            val listener = object : BuildListener {
                override fun stdout(line: String) {
                    client.logMessage(MessageParams(MessageType.Log, line))
                }

                override fun stderr(line: String) = stdout(line)
            }
            val (status, allErrors) = BuildEngine.build(parent.uri, config, listener)

            for (document in workspace.documents.filterIsInstance<LatexDocument>()) {
                val diagnostics = PublishDiagnosticsParams(document.uri.toString(), emptyList())
                client.publishDiagnostics(diagnostics)
            }

            for ((uri, errors) in allErrors.groupBy { it.uri }) {
                val diagnostics = PublishDiagnosticsParams(uri.toString(), errors.map { it.toDiagnostic() })
                client.publishDiagnostics(diagnostics)
            }

            client.setStatus(StatusParams(ServerStatus.IDLE))
            status
        }
    }
}
