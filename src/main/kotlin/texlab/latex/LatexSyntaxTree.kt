package texlab.latex

class LatexSyntaxTree(private val root: LatexDocumentSyntax) {

    constructor(text: String) : this(LatexParser.parse(text))
}
