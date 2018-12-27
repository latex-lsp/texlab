package texlab.build

import java.io.IOException
import java.net.URI
import java.nio.file.Files
import java.nio.file.Paths

object BuildEngine {
    fun build(uri: URI, config: BuildConfig): BuildResult {
        if (uri.scheme != "file") {
            return BuildResult(BuildStatus.FAILURE, emptyList())
        }

        val texPath = Paths.get(uri)
        val command = listOf(config.executable, *config.args.toTypedArray(), texPath.toString())
        return try {
            val exitCode = ProcessBuilder(command)
                    .directory(texPath.parent.toFile())
                    .start()
                    .waitFor()

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
