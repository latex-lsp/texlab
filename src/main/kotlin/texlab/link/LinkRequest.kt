package texlab.link

import texlab.Document
import texlab.Workspace
import java.net.URI

class LinkRequest(val workspace: Workspace,
                  val uri: URI) {
    val document: Document = workspace.documents.first { it.uri == uri }
}
