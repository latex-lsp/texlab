package texlab

import org.eclipse.lsp4j.DocumentLink
import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.FoldingRange
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

    override fun foldingRange(): List<FoldingRange> {
        // TODO
        return emptyList()
    }
}
