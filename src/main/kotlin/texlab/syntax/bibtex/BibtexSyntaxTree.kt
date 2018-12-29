package texlab.syntax.bibtex

class BibtexSyntaxTree(text: String) {
    val root: BibtexDocumentSyntax = BibtexParser.parse(text)
}
