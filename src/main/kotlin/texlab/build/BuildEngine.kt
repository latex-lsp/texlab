package texlab.build

import org.eclipse.lsp4j.jsonrpc.CancelChecker
import java.io.IOException
import java.net.URI
import java.nio.file.Files
import java.nio.file.Paths
import java.util.concurrent.CancellationException
import java.util.concurrent.TimeUnit

object BuildEngine {
    fun build(uri: URI, config: BuildConfig, cancelChecker: CancelChecker): BuildResult {
        val texFile = Paths.get(uri).toFile()
        val command = listOf(config.executable, *config.args.toTypedArray(), texFile.absolutePath)
        return try {
            val buildLogFile = Paths.get(texFile.parent, "texlab-build.log").toFile()
            if (buildLogFile.exists()) {
                buildLogFile.delete()
            }

            val process = ProcessBuilder(command)
                    .directory(texFile.parentFile)
                    .redirectOutput(buildLogFile)
                    .redirectErrorStream(true)
                    .start()

            try {
                while (!process.waitFor(500, TimeUnit.MILLISECONDS)) {
                    cancelChecker.checkCanceled()
                }
            } catch (e: CancellationException) {
                process.destroy()
                throw e
            }

            val status = if (process.exitValue() == 0) {
                BuildStatus.SUCCESS
            } else {
                BuildStatus.ERROR
            }

            val logPath = Paths.get(texFile.parent, texFile.nameWithoutExtension + ".log")
            val log = Files.readAllBytes(logPath).toString(Charsets.UTF_8)
            val errors = BuildErrorParser.parse(uri, log)
            BuildResult(status, errors)
        } catch (e: IOException) {
            BuildResult(BuildStatus.FAILURE, emptyList())
        }
    }
}
