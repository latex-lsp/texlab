package texlab.syntax.latex

data class LatexLabelDefinition(val command: LatexCommandSyntax) {

    val name: LatexToken
        get() = (command.args[0].children[0] as LatexTextSyntax).words[0]

    companion object {
        fun analyze(root: LatexSyntaxNode): List<LatexLabelDefinition> {
            return root.descendants()
                    .filterIsInstance<LatexCommandSyntax>()
                    .filter { it.name.text == "\\label" }
                    .mapNotNull { analyze(it) }
        }

        private fun analyze(command: LatexCommandSyntax): LatexLabelDefinition? {
            return if (command.extractWord(0) == null) {
                null
            } else {
                LatexLabelDefinition(command)
            }
        }
    }
}
