package texlab.syntax.latex

data class LatexSection(val command: LatexCommandSyntax, val text: String, val level: Int) {
    companion object {
        private val COMMAND_NAMES = arrayOf(
                "\\chapter", "\\chapter*",
                "\\section", "\\section*",
                "\\subsection", "\\subsection*",
                "\\subsubsection", "\\subsubsection*",
                "\\paragraph", "\\paragraph*",
                "\\subparagraph", "\\subparagraph*")

        fun find(root: LatexSyntaxNode): List<LatexSection> {
            return root.descendants()
                    .filterIsInstance<LatexCommandSyntax>()
                    .filter { COMMAND_NAMES.contains(it.name.text) }
                    .mapNotNull { analyze(it) }
        }

        private fun analyze(command: LatexCommandSyntax): LatexSection? {
            val text = command.extractText(0) ?: return null
            val level = COMMAND_NAMES.indexOf(command.name.text) / 2
            return LatexSection(command, text.words.joinToString(" ") { it.text }, level)
        }
    }
}
