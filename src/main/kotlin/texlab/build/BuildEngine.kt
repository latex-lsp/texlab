package texlab.build

import java.io.IOException
import java.net.URI
import java.nio.file.Files
import java.nio.file.Paths

object BuildEngine {
    fun build(uri: URI, config: BuildConfig): BuildResult {
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

            val exitCode = process.waitFor()
            val status = if (exitCode == 0) {
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
