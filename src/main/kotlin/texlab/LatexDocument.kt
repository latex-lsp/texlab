package texlab

import texlab.latex.LatexSyntaxTree
import java.net.URI

class LatexDocument(uri: URI) : Document(uri) {
    var tree: LatexSyntaxTree = LatexSyntaxTree(text)

    override fun analyze() {
        tree = LatexSyntaxTree(text)
    }
}
