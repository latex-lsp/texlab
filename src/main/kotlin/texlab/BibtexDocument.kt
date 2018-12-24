package texlab

import org.eclipse.lsp4j.*
import java.net.URI

class BibtexDocument(uri: URI) : Document(uri) {

    override fun analyze() {
        // TODO
    }

    override fun documentSymbol(workspace: Workspace): List<DocumentSymbol> {
        // TODO
        return emptyList()
    }

    override fun documentLink(workspace: Workspace): List<DocumentLink> {
        // TODO
        return emptyList()
    }

    override fun rename(workspace: Workspace, position: Position, newName: String): WorkspaceEdit? {
        // TODO
        return null
    }

    override fun foldingRange(): List<FoldingRange> {
        // TODO
        return emptyList()
    }
}
