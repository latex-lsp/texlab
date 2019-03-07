package texlab.diagnostics

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.future.await
import kotlinx.coroutines.withContext
import org.eclipse.lsp4j.Diagnostic
import org.eclipse.lsp4j.DiagnosticSeverity
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import java.io.IOException
import java.io.InputStream
import java.util.concurrent.CompletableFuture

object LatexLinter {
    private val LINE_REGEX = Regex("""(\d+):(\d+):(\d+):(\w+):(\w)+:(.*)""")

    suspend fun lint(text: String): List<Diagnostic> {
        try {
            val process = withContext(Dispatchers.IO) {
                ProcessBuilder("chktex", "-I0", "-f%l:%c:%d:%k:%n:%m\n")
                        .redirectOutput(ProcessBuilder.Redirect.PIPE)
                        .redirectError(ProcessBuilder.Redirect.PIPE)
                        .redirectInput(ProcessBuilder.Redirect.PIPE)
                        .start()
            }

            val stderr = readTextAsync(process.errorStream)
            val stdout = readTextAsync(process.inputStream)
            withContext(Dispatchers.IO) {
                process.outputStream.write(text.toByteArray())
                process.outputStream.flush()
                process.outputStream.close()
                process.waitFor()
            }

            stderr.await()
            return stdout.await()
                    .lines()
                    .filter { it.isNotBlank() }
                    .mapNotNull { parse(it) }
        } catch (e: IOException) {
            return emptyList()
        }
    }

    private fun readTextAsync(stream: InputStream) = CompletableFuture.supplyAsync {
        stream.bufferedReader(Charsets.UTF_8).readText()
    }

    private fun parse(text: String): Diagnostic? {
        val match = LINE_REGEX.matchEntire(text) ?: return null
        val line = match.groupValues[1].toInt() - 1
        val character = match.groupValues[2].toInt() - 1
        val digit = match.groupValues[3].toInt()
        val kind = match.groupValues[4]
        val number = match.groupValues[5]
        val message = match.groupValues[6]
        val range = Range(Position(line, character), Position(line, character + digit))
        val severity = when (kind) {
            "Message" -> DiagnosticSeverity.Information
            "Warning" -> DiagnosticSeverity.Warning
            else -> DiagnosticSeverity.Error
        }
        return Diagnostic(range, message, severity, "chktex", number)
    }
}
