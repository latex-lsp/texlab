package texlab.syntax.latex

data class LatexLabel(val command: LatexCommandSyntax) {

    val name: LatexToken
        get() = (command.args[0].children[0] as LatexTextSyntax).words[0]

    companion object {
        fun findDefinitions(root: LatexSyntaxNode): List<LatexLabel> {
            return root.descendants()
                    .filterIsInstance<LatexCommandSyntax>()
                    .filter { it.name.text == "\\label" }
                    .mapNotNull { analyze(it) }
        }

        fun findReferences(root: LatexSyntaxNode): List<LatexLabel> {
            return root.descendants()
                    .filterIsInstance<LatexCommandSyntax>()
                    .filter { it.name.text == "\\ref" || it.name.text == "\\autoref" }
                    .mapNotNull { analyze(it) }
        }

        private fun analyze(command: LatexCommandSyntax): LatexLabel? {
            return if (command.extractWord(0) == null) {
                null
            } else {
                LatexLabel(command)
            }
        }
    }
}
