package texlab.completion.latex.data

import java.io.IOException
import java.nio.file.Files
import java.nio.file.Paths
import java.util.concurrent.TimeUnit

object LatexCompiler {
    private const val TIMEOUT: Long = 10

    fun compile(code: String, format: LatexFormat): String? {
        val directory = createTempDir()
        directory.deleteOnExit()

        val file = Paths.get(directory.absolutePath, "code.tex")
        Files.write(file, code.toByteArray())

        val executable = when (format) {
            LatexFormat.LATEX -> "latex"
            LatexFormat.LUALATEX -> "lualatex"
            LatexFormat.XELATEX -> "xelatex"
        }

        val process = ProcessBuilder(executable, "-interaction=batchmode", "-shell-escape", "code.tex")
                .directory(directory)
                .start()

        process.waitFor(TIMEOUT, TimeUnit.SECONDS)
        process.destroy()
        process.waitFor()

        return try {
            val logFile = Paths.get(directory.absolutePath, "code.log")
            Files.readAllBytes(logFile).toString(Charsets.UTF_8)
        } catch (e: IOException) {
            null
        }
    }
}

