package texlab.diagnostics

import texlab.Document
import java.net.URI

data class DiagnosticsRequest(val uri: URI,
                              val relatedDocuments: List<Document>) {
    val document: Document = relatedDocuments.first { it.uri == uri }
}
