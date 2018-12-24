package texlab.link

import texlab.Document
import texlab.Workspace
import java.net.URI

class LinkRequest(val uri: URI,
                  val workspace: Workspace) {
    val document: Document = workspace.documents.first { it.uri == uri }
}
