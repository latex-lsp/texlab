package texlab.syntax.latex

data class LatexCitation(val command: LatexCommandSyntax) {
    val name: LatexToken
        get() = (command.args[0].children[0] as LatexTextSyntax).words[0]

    companion object {
        val COMMAND_NAMES = listOf(
                "\\cite", "\\nocite", "\\citet", "\\citep", "\\citet*", "\\citep*",
                "\\citeauthor", "\\citeauthor*", "\\citeyear", "\\citeyearpar",
                "\\citealt", "\\citealp", "\\citetext")

        fun find(root: LatexSyntaxNode): List<LatexCitation> {
            return root.descendants()
                    .filterIsInstance<LatexCommandSyntax>()
                    .filter { COMMAND_NAMES.contains(it.name.text) }
                    .mapNotNull { analyze(it) }
        }

        private fun analyze(command: LatexCommandSyntax): LatexCitation? {
            return if (command.extractWord(0) == null) {
                null
            } else {
                LatexCitation(command)
            }
        }
    }
}
