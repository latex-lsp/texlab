package texlab

import kotlinx.coroutines.*
import org.eclipse.lsp4j.DidChangeConfigurationParams
import org.eclipse.lsp4j.DidChangeWatchedFilesParams
import org.eclipse.lsp4j.services.WorkspaceService
import texlab.build.BuildErrorParser
import java.io.File
import java.io.IOException
import java.nio.file.Files
import kotlin.coroutines.CoroutineContext

@ObsoleteCoroutinesApi
class WorkspaceServiceImpl(private val service: TextDocumentServiceImpl) : WorkspaceService, CoroutineScope {
    override val coroutineContext: CoroutineContext = Dispatchers.Default + SupervisorJob()

    override fun didChangeWatchedFiles(params: DidChangeWatchedFilesParams) {
        launch {
            for (change in params.changes) {
                val logPath = File(URIHelper.parse(change.uri)).toPath()
                val texPath = logPath.resolveSibling(logPath.toFile().nameWithoutExtension + ".tex")
                val texUri = texPath.toUri()

                service.workspaceActor.withWorkspace { workspace ->
                    val document = workspace.documents.firstOrNull { it.uri == texUri }
                    if (document != null) {
                        try {
                            val log = withContext(Dispatchers.IO) {
                                Files.readAllBytes(logPath).toString(Charsets.UTF_8)
                            }
                            val allErrors = BuildErrorParser.parse(texUri, log)

                            service.buildDiagnosticsProvider.diagnosticsByUri = allErrors
                                    .groupBy { it.uri }
                                    .mapValues { errors -> errors.value.map { it.toDiagnostic() } }

                            workspace.documents.forEach { service.publishDiagnostics(it.uri) }
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
}
