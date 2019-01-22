package texlab.build

import org.eclipse.lsp4j.jsonrpc.CancelChecker
import texlab.ProgressListener
import texlab.ProgressParams
import java.io.IOException
import java.net.URI
import java.nio.file.Paths
import java.util.concurrent.CancellationException
import java.util.concurrent.TimeUnit

object BuildEngine {
    fun build(uri: URI, config: BuildConfig, cancelChecker: CancelChecker, listener: ProgressListener?): BuildStatus {
        val texFile = Paths.get(uri).toFile()
        val progressParams = ProgressParams("build", "Building...", texFile.name)
        listener?.onReportProgress(progressParams)

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

            if (process.exitValue() == 0) {
                BuildStatus.SUCCESS
            } else {
                BuildStatus.ERROR
            }
        } catch (e: IOException) {
            BuildStatus.FAILURE
        } finally {
            listener?.onReportProgress(progressParams.copy(done = true))
        }
    }
}
