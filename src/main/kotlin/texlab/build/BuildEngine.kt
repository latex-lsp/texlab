package texlab.build

import java.io.IOException
import java.io.InputStream
import java.net.URI
import java.nio.file.Files
import java.nio.file.Paths
import java.util.concurrent.CompletableFuture

object BuildEngine {
    fun build(uri: URI, config: BuildConfig, listener: BuildListener? = null): BuildResult {
        if (uri.scheme != "file") {
            return BuildResult(BuildStatus.FAILURE, emptyList())
        }

        val texPath = Paths.get(uri)
        val command = listOf(config.executable, *config.args.toTypedArray(), texPath.toString())
        return try {
            val process = ProcessBuilder(command)
                    .directory(texPath.parent.toFile())
                    .redirectOutput(ProcessBuilder.Redirect.PIPE)
                    .redirectError(ProcessBuilder.Redirect.PIPE)
                    .start()

            fun readTextAsync(stream: InputStream, action: (line: String) -> Unit): CompletableFuture<Unit> {
                return CompletableFuture.supplyAsync {
                    stream.reader(Charsets.UTF_8).forEachLine { line ->
                        action(line)
                    }
                }
            }

            readTextAsync(process.inputStream) { listener?.stdout(it) }
            readTextAsync(process.errorStream) { listener?.stderr(it) }

            val exitCode = process.waitFor()
            val status = if (exitCode == 0) {
                BuildStatus.SUCCESS
            } else {
                BuildStatus.ERROR
            }

            val logPath = Paths.get(texPath.parent.toString(), texPath.toFile().nameWithoutExtension + ".log")
            val log = Files.readAllBytes(logPath).toString(Charsets.UTF_8)
            val errors = BuildErrorParser.parse(uri, log)
            BuildResult(status, errors)
        } catch (e: IOException) {
            BuildResult(BuildStatus.FAILURE, emptyList())
        }
    }
}
