package texlab

import org.eclipse.lsp4j.DocumentSymbol
import java.net.URI

class BibtexDocument(uri: URI) : Document(uri) {

    override fun analyze() {
        // TODO
    }

    override fun documentSymbol(workspace: Workspace): List<DocumentSymbol> {
        // TODO
        return emptyList()
    }
}
