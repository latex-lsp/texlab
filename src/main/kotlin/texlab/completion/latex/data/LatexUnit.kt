package texlab.completion.latex.data

import texlab.resolver.LatexResolver
import java.io.File

data class LatexUnit(val file: File,
                     val kind: LatexUnitKind,
                     val format: LatexFormat,
                     val references: List<File>,
                     val likelyPrimitives: Set<String>) {
    fun checkPrimitives(candidates: Iterable<String>): LatexPrimitives {
        val testCode = buildString {
            appendln(buildCodeHeader(file.nameWithoutExtension, kind))
            appendln("\\usepackage{etoolbox}")
            appendln("\\begin{document}")

            for (candidate in candidates) {
                appendln("\\ifcsundef{$candidate}{} {")
                appendln("\\ifcsundef{end$candidate}")
                appendln("{ \\wlog{cmd:$candidate} }")
                appendln("{ \\wlog{env:$candidate} } }")
            }

            appendln("\\end{document}")
        }

        val log = LatexCompiler.compile(testCode, format) ?: ""
        val commands = mutableListOf<String>()
        val environments = mutableListOf<String>()
        for (line in log.lines()) {
            if (line.startsWith("cmd:")) {
                commands.add(line.split(':')[1])
            } else if (line.startsWith("env:")) {
                environments.add(line.split(':')[1])
            }
        }

        return LatexPrimitives(commands, environments)
    }

    companion object {
        private val fileRegex = Regex("""[a-zA-Z0-9_\-.]+\.(sty|tex|def|cls)""")
        private val primitiveRegex = Regex("""[a-zA-Z]+""")

        fun load(file: File, resolver: LatexResolver): LatexUnit? {
            val kind = when (file.extension) {
                "cls" -> LatexUnitKind.CLS
                else -> LatexUnitKind.STY
            }

            val format = when {
                file.absolutePath.contains("lua") -> LatexFormat.LUALATEX
                file.absolutePath.contains("xe") -> LatexFormat.XELATEX
                else -> LatexFormat.LATEX
            }

            val testCode = buildString {
                appendln(buildCodeHeader(file.nameWithoutExtension, kind))
                appendln("\\listfiles")
                appendln("\\begin{document} \\end{document}")
            }
            val log = LatexCompiler.compile(testCode, format) ?: return null

            val includes = extractIncludes(log, kind, resolver)
            val references = includes.filter { it.extension == "sty" && it != file }
            val likelyPrimitives = getLikelyPrimitives(includes)
            return LatexUnit(file, kind, format, references, likelyPrimitives)
        }

        private fun getLikelyPrimitives(includes: Iterable<File>): Set<String> {
            return includes.joinToString(System.lineSeparator()) { it.readText() }
                    .let { primitiveRegex.findAll(it) }
                    .map { it.value }
                    .toHashSet()
        }

        private fun extractIncludes(log: String, kind: LatexUnitKind, resolver: LatexResolver): List<File> {
            val startIndex = log.indexOf("*File List*")
            if (startIndex < 0) {
                return listOf()
            }

            return fileRegex.findAll(log, startIndex)
                    .map { it.value }
                    .filterNot { it == "article.cls" && kind != LatexUnitKind.CLS }
                    .mapNotNull { resolver.filesByName[it] }
                    .toList()
        }

        private fun buildCodeHeader(name: String, kind: LatexUnitKind): String = buildString {
            when (kind) {
                LatexUnitKind.STY -> appendln("\\documentclass{article} \\usepackage{$name}")
                LatexUnitKind.CLS -> appendln("\\documentclass{$name}")
            }
        }
    }
}
