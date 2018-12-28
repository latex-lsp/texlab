package texlab.syntax.latex

data class LatexInclude(val command: LatexCommandSyntax, val path: String) {
    companion object {
        private val COMMAND_NAMES =
                arrayOf("\\include", "\\input", "\\bibliography",
                        "\\addbibresource", "\\usepackage", "\\documentclass")

        fun analyze(root: LatexSyntaxNode): List<LatexInclude> {
            return root.descendants()
                    .filterIsInstance<LatexCommandSyntax>()
                    .filter { COMMAND_NAMES.contains(it.name.text) }
                    .mapNotNull { analyze(it) }
        }

        private fun analyze(command: LatexCommandSyntax): LatexInclude? {
            val text = command.extractText(0) ?: return null
            val path = text.words.joinToString(" ") { it.text }
            return LatexInclude(command, path)
        }
    }
}
