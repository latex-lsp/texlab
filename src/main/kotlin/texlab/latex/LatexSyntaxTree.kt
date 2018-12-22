package texlab.latex

class LatexSyntaxTree(text: String) {

    private val root: LatexDocumentSyntax = LatexParser.parse(text)

    val includes: List<LatexInclude> = LatexInclude.analyze(root)
}
