package texlab.build

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import texlab.ProgressListener
import texlab.ProgressParams
import java.io.IOException
import java.net.URI
import java.nio.file.Paths

object BuildEngine {
    suspend fun build(uri: URI, config: BuildConfig, listener: ProgressListener?): BuildStatus {
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
                    .directory(texFile.parentFile)
                    .redirectOutput(buildLogFile)
                    .redirectErrorStream(true)
                    .start()

            try {
                withContext(Dispatchers.Default) {
                    while (process.isAlive) {
                        delay(250)
                    }
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
