package texlab.latex

import org.eclipse.lsp4j.DocumentLink
import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.WorkspaceEdit
import texlab.Document
import texlab.Workspace
import java.net.URI

class LatexDocument(uri: URI) : Document(uri) {

    var tree: LatexSyntaxTree = LatexSyntaxTree(text)

    override fun analyze() {
        tree = LatexSyntaxTree(text)
    }

    override fun documentSymbol(workspace: Workspace): List<DocumentSymbol> {
        return LatexEnvironmentSymbolFinder.find(tree)
    }

    override fun rename(workspace: Workspace, position: Position, newName: String): WorkspaceEdit? {
        val edits = LatexEnvironmentRenamer.rename(tree, position, newName)
        return WorkspaceEdit(mutableMapOf(uri.toString() to edits))
    }

    override fun documentLink(workspace: Workspace): List<DocumentLink> {
        return tree.includes.mapNotNull {
            val range = it.command.args[0].children[0].range
            val target = workspace.resolve(uri, it.path) ?: return@mapNotNull null
            DocumentLink(range, target.uri.toString())
        }
    }
}
