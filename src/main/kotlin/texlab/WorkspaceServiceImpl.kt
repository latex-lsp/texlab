package texlab

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.sync.withLock
import org.eclipse.lsp4j.DidChangeConfigurationParams
import org.eclipse.lsp4j.DidChangeWatchedFilesParams
import org.eclipse.lsp4j.services.WorkspaceService
import texlab.build.BuildErrorParser
import java.io.File
import java.io.IOException
import java.nio.file.Files
import kotlin.coroutines.CoroutineContext

class WorkspaceServiceImpl(override val coroutineContext: CoroutineContext,
                           private val service: TextDocumentServiceImpl) : WorkspaceService, CoroutineScope {
    override fun didChangeWatchedFiles(params: DidChangeWatchedFilesParams) = runBlocking {
        for (change in params.changes) {
            val logPath = File(URIHelper.parse(change.uri)).toPath()
            val texPath = logPath.resolveSibling(logPath.toFile().nameWithoutExtension + ".tex")
            val texUri = texPath.toUri()
            val document = service.workspace.documents.firstOrNull { it.uri == texUri }
            if (document != null) {
                try {
                    val log = Files.readAllBytes(logPath).toString(Charsets.UTF_8)
                    val allErrors = BuildErrorParser.parse(texUri, log)

                    service.buildDiagnosticsProvider.diagnosticsByUri = allErrors
                            .groupBy { it.uri }
                            .mapValues { errors -> errors.value.map { it.toDiagnostic() } }

                    service.workspace.withLock {
                        service.workspace.documents.forEach { service.publishDiagnostics(it.uri) }
                    }
                } catch (e: IOException) {
                    // File is still locked
                }
            }
        }
    }

    override fun didChangeConfiguration(params: DidChangeConfigurationParams) {
    }
}
