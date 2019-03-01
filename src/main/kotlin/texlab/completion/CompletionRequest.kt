package texlab.completion

import org.eclipse.lsp4j.Position
import texlab.Document
import texlab.Workspace
import java.net.URI

data class CompletionRequest(val uri: URI,
                             val position: Position,
                             val workspace: Workspace) {
    val relatedDocuments = workspace.relatedDocuments(uri)
    val document: Document = relatedDocuments.first { it.uri == uri }
}
