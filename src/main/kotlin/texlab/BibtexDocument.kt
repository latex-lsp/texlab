package texlab

import org.eclipse.lsp4j.DocumentLink
import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.WorkspaceEdit
import java.net.URI

class BibtexDocument(uri: URI) : Document(uri) {

    override fun analyze() {
        // TODO
    }

    override fun documentSymbol(workspace: Workspace): List<DocumentSymbol> {
        // TODO
        return listOf()
    }

    override fun documentLink(workspace: Workspace): List<DocumentLink> {
        // TODO
        return listOf()
    }

    override fun rename(workspace: Workspace, position: Position, newName: String): WorkspaceEdit? {
        // TODO
        return null
    }
}
