package texlab.search

import java.io.File
import java.io.IOException
import java.nio.file.Paths

object ForwardSearchTool {
    fun search(file: File, parent: File, lineNumber: Int, config: ForwardSearchConfig): ForwardSearchResult {
        val pdfFile = Paths.get(parent.parent, parent.nameWithoutExtension + ".pdf").toString()

        fun replacePlaceholder(argument: String): String {
            return if (argument.startsWith('"') && argument.endsWith('"')) {
                argument
            } else {
                argument
                        .replace("%f", file.path)
                        .replace("%p", pdfFile)
                        .replace("%l", lineNumber.toString())
            }
        }

        if (config.executable == null) {
            return ForwardSearchResult(ForwardSearchStatus.UNCONFIGURED)
        }

        val args = config.args
                .map { replacePlaceholder(it) }
                .toTypedArray()

        val command = listOf(config.executable, *args)
        val status = try {
            ProcessBuilder(command)
                    .redirectOutput(ProcessBuilder.Redirect.PIPE)
                    .redirectError(ProcessBuilder.Redirect.PIPE)
                    .start()
            ForwardSearchStatus.SUCCESS
        } catch (e: IOException) {
            ForwardSearchStatus.FAILURE
        }
        return ForwardSearchResult(status)
    }
}
