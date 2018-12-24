package texlab.rename

import org.eclipse.lsp4j.Position
import texlab.Document
import java.net.URI

class RenameRequest(val uri: URI,
                    val relatedDocuments: List<Document>,
                    val position: Position,
                    val newName: String) {
    val document: Document = relatedDocuments.first { it.uri == uri }
}

