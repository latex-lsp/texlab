package texlab.completion.latex.data

import java.io.File
import java.io.IOException
import java.nio.file.Files
import java.nio.file.Paths

data class LatexResolver(val filesByName: Map<String, File>) {
    companion object {
        fun create(): LatexResolver {
            try {
                val process = ProcessBuilder("kpsewhich", "-var-value", "TEXMFDIST")
                        .redirectOutput(ProcessBuilder.Redirect.PIPE)
                        .start()
                process.waitFor()

                val directory = process.inputStream.bufferedReader().readLine()
                val databaseFile = Paths.get(directory, "ls-R")
                val lines = Files.readAllLines(databaseFile)
                val filesByName = parseDatabase(directory, lines).associateBy { it.name }
                return LatexResolver(filesByName)
            } catch (e: IOException) {
                throw TexDistributionNotFoundException("Could not execute kpsewhich.")
            }
        }

        private fun parseDatabase(directory: String, lines: Iterable<String>) = sequence {
            val validDirectories = listOf("plain", "generic", "latex", "luatex", "lualatex", "xetex", "xelatex")
                    .map { Paths.get(directory, "tex", it) }

            var currentDirectory = Paths.get("")
            for (line in lines.filter { it.isNotBlank() && !it.startsWith("%") }) {
                if (line.endsWith(":")) {
                    val path = line.substring(0, line.length - 1)
                    currentDirectory = Paths.get(directory, path).normalize()
                } else if (validDirectories.any { currentDirectory.startsWith(it) }) {
                    val file = Paths.get(currentDirectory.toString(), line).toFile()
                    if (file.extension.isNotBlank()) {
                        yield(file)
                    }
                }
            }
        }
    }
}
