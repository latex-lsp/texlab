package texlab.syntax.latex

class LatexSyntaxTree(text: String) {
    val root: LatexDocumentSyntax = LatexParser.parse(text)

    val includes: List<LatexInclude> = LatexInclude.analyze(root)

    val components: List<String> = includes.mapNotNull {
        when (it.command.name.text) {
            "\\usepackage" -> it.path + ".sty"
            "\\documentclass" -> it.path + ".cls"
            else -> null
        }
    }

    val environments: List<LatexEnvironment> = LatexEnvironment.analyze(root)

    val sections: List<LatexSection> = LatexSection.analyze(root)

    val labelDefinitions: List<LatexLabel> = LatexLabel.findDefinitions(root)

    val labelReferences: List<LatexLabel> = LatexLabel.findReferences(root)

    val isStandalone: Boolean = environments.any { it.beginName == "document" || it.endName == "document" }
}
