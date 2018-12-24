package texlab.rename

import org.eclipse.lsp4j.TextEdit
import org.eclipse.lsp4j.WorkspaceEdit
import texlab.LatexDocument
import texlab.contains

object LatexEnvironmentRenamer : Renamer {
    override fun rename(request: RenameRequest): WorkspaceEdit? {
        if (request.document !is LatexDocument) {
            return null
        }

        for (environment in request.document.tree.environments) {
            val begin = environment.beginNameRange.contains(request.position)
            val end = environment.endNameRange.contains(request.position)
            if (begin || end) {
                val edits = listOf(
                        TextEdit(environment.beginNameRange, request.newName),
                        TextEdit(environment.endNameRange, request.newName))
                return WorkspaceEdit(mutableMapOf(request.uri.toString() to edits))
            }
        }
        return null
    }
}
