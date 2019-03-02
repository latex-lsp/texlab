package texlab.provider

import texlab.Workspace
import java.net.URI

data class FeatureRequest<T>(val uri: URI,
                             val workspace: Workspace,
                             val params: T) {
    val relatedDocuments = workspace.relatedDocuments(uri)
    val document = relatedDocuments.first { it.uri == uri }
}
