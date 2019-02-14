package texlab.search

import java.io.File
import java.io.IOException
import java.nio.file.Paths
import java.util.concurrent.TimeUnit

object ForwardSearchTool {
    fun search(file: File, parent: File, lineNumber: Int, config: ForwardSearchConfig): ForwardSearchResult {
        val pdfFile = Paths.get(parent.parent, parent.nameWithoutExtension + ".pdf").toString()

        fun replacePlaceholder(argument: String): String {
            return argument
                    .replace("%f", file.path)
                    .replace("%p", pdfFile)
                    .replace("%l", lineNumber.toString())
        }

        if (config.executable == null) {
            return ForwardSearchResult(ForwardSearchStatus.UNCONFIGURED)
        }

        val args = config.args
                .map { replacePlaceholder(it) }
                .toTypedArray()
        val command = listOf(config.executable, *args)
        val status = try {
            val process = ProcessBuilder(command)
                    .redirectOutput(ProcessBuilder.Redirect.PIPE)
                    .redirectError(ProcessBuilder.Redirect.PIPE)
                    .start()
            if (process.waitFor(5, TimeUnit.SECONDS)) {
                if (process.exitValue() == 0) {
                    ForwardSearchStatus.SUCCESS
                } else {
                    ForwardSearchStatus.ERROR
                }
            } else {
                ForwardSearchStatus.SUCCESS
            }

        } catch (e: IOException) {
            ForwardSearchStatus.FAILURE
        }
        return ForwardSearchResult(status)
    }
}
