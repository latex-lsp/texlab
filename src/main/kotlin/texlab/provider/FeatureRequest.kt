package texlab.provider

import org.slf4j.Logger
import texlab.Workspace
import java.net.URI

data class FeatureRequest<T>(val uri: URI,
                             val workspace: Workspace,
                             val params: T,
                             val logger: Logger) {
    val relatedDocuments = workspace.relatedDocuments(uri)
    val document = relatedDocuments.first { it.uri == uri }
}
