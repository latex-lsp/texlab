package texlab.forwardSearch

import java.io.File
import java.io.IOException
import java.nio.file.Paths

object ForwardSearchTool {
    fun search(file: File,
               parent: File,
               lineNumber: Int,
               config: ForwardSearchConfig): ForwardSearchStatus {
        val pdfFile = Paths.get(parent.parent, parent.nameWithoutExtension + ".pdf").toString()

        fun replacePlaceholder(argument: String): String {
            return argument
                    .replace("%f", file.path)
                    .replace("%p", pdfFile)
                    .replace("%l", lineNumber.toString())
        }

        if (config.executable == null) {
            return ForwardSearchStatus.UNCONFIGURED
        }

        val args = config.args
                .map { replacePlaceholder(it) }
                .toTypedArray()
        val command = listOf(config.executable, *args)
        return try {
            val process = ProcessBuilder(command)
                    .redirectOutput(ProcessBuilder.Redirect.PIPE)
                    .redirectError(ProcessBuilder.Redirect.PIPE)
                    .start()
            process.waitFor()
            ForwardSearchStatus.SUCCESS
        } catch (e: IOException) {
            ForwardSearchStatus.ERROR
        }
    }
}
