package texlab.rename

import org.eclipse.lsp4j.RenameParams
import org.eclipse.lsp4j.TextEdit
import org.eclipse.lsp4j.WorkspaceEdit
import texlab.LatexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest

object LatexEnvironmentRenameProvider : FeatureProvider<RenameParams, WorkspaceEdit?> {
    override suspend fun get(request: FeatureRequest<RenameParams>): WorkspaceEdit? {
        if (request.document !is LatexDocument) {
            return null
        }

        for (environment in request.document.tree.environments) {
            val begin = environment.beginNameRange.contains(request.params.position)
            val end = environment.endNameRange.contains(request.params.position)
            if (begin || end) {
                val edits = listOf(
                        TextEdit(environment.beginNameRange, request.params.newName),
                        TextEdit(environment.endNameRange, request.params.newName))
                return WorkspaceEdit(mapOf(request.uri.toString() to edits))
            }
        }

        return null
    }
}
