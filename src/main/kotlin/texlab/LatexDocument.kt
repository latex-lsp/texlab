package texlab

import org.eclipse.lsp4j.DocumentSymbol
import texlab.syntax.latex.LatexEnvironmentSymbolFinder
import texlab.syntax.latex.LatexSyntaxTree
import java.net.URI

class LatexDocument(uri: URI) : Document(uri) {

    var tree: LatexSyntaxTree = LatexSyntaxTree(text)

    override fun analyze() {
        tree = LatexSyntaxTree(text)
    }

    override fun documentSymbol(workspace: Workspace): List<DocumentSymbol> {
        return LatexEnvironmentSymbolFinder.find(tree)
    }
}
