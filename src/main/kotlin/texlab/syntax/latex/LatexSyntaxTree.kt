package texlab.syntax.latex

class LatexSyntaxTree(text: String) {

    val root: LatexDocumentSyntax = LatexParser.parse(text)

    val includes: List<LatexInclude> = LatexInclude.analyze(root)

    val environments: List<LatexEnvironment> = LatexEnvironment.analyze(root)

    val sections: List<LatexSection> = LatexSection.analyze(root)

    val labelDefinitions: List<LatexLabel> = LatexLabel.findDefinitions(root)

    val labelReferences: List<LatexLabel> = LatexLabel.findReferences(root)
}
