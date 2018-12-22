package texlab

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.WorkspaceEdit
import texlab.latex.LatexEnvironmentRenamer
import texlab.latex.LatexEnvironmentSymbolFinder
import texlab.latex.LatexSyntaxTree
import java.net.URI

class LatexDocument(uri: URI) : Document(uri) {

    var tree: LatexSyntaxTree = LatexSyntaxTree(text)

    override fun analyze() {
        tree = LatexSyntaxTree(text)
    }

    override fun documentSymbol(): List<DocumentSymbol> {
        return LatexEnvironmentSymbolFinder.find(tree)
    }

    override fun rename(documents: List<Document>, position: Position, newName: String): WorkspaceEdit? {
        val edits = LatexEnvironmentRenamer.rename(tree, position, newName)
        return WorkspaceEdit(mutableMapOf(uri.toString() to edits))
    }
}
