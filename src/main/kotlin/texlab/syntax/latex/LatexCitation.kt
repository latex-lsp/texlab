package texlab.syntax.latex

data class LatexCitation(val command: LatexCommandSyntax) {
    val name: LatexToken
        get() = (command.args[0].children[0] as LatexTextSyntax).words[0]

    companion object {
        val COMMAND_NAMES = listOf(
                "\\cite", "\\cite*", "\\Cite", "\\nocite", "\\citet", "\\citep", "\\citet*", "\\citep*",
                "\\citeauthor", "\\citeauthor*", "\\Citeauthor", "\\Citeauthor*", "\\citetitle", "\\citetitle*",
                "\\citeyear", "\\citeyear*", "\\citedate", "\\citedate*", "\\citeurl", "\\fullcite",
                "\\citeyearpar", "\\citealt", "\\citealp", "\\citetext", "\\parencite", "\\parencite*",
                "\\Parencite", "\\footcite", "\\footfullcite", "\\footcitetext", "\\textcite", "\\Textcite",
                "\\smartcite", "\\Smartcite", "\\supercite", "\\autocite", "\\Autocite", "\\autocite*",
                "\\Autocite*", "\\volcite", "\\Volcite", "\\pvolcite", "\\Pvolcite", "\\fvolcite", "\\ftvolcite",
                "\\svolcite", "\\Svolcite", "\\tvolcite", "\\Tvolcite", "\\avolcite", "\\Avolcite", "\\notecite",
                "\\notecite", "\\pnotecite", "\\Pnotecite", "\\fnotecite")

        fun find(root: LatexSyntaxNode): List<LatexCitation> {
            return root.descendants
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
