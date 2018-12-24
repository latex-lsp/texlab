package texlab.rename

import org.eclipse.lsp4j.TextEdit
import org.eclipse.lsp4j.WorkspaceEdit
import texlab.LatexDocument
import texlab.contains

object LatexLabelRenamer : Renamer {
    override fun rename(request: RenameRequest): WorkspaceEdit? {
        if (request.document !is LatexDocument) {
            return null
        }

        val label = request.document.tree
                .labelReferences
                .plus(request.document.tree.labelDefinitions)
                .firstOrNull { it.name.range.contains(request.position) }
                ?: return null

        val changes = mutableMapOf<String, List<TextEdit>>()
        for (document in request.relatedDocuments.filterIsInstance<LatexDocument>()) {
            val edits = document.tree.labelReferences
                    .plus(document.tree.labelDefinitions)
                    .filter { it.name.text == label.name.text }
                    .map { TextEdit(it.name.range, request.newName) }
            changes[document.uri.toString()] = edits
        }
        return WorkspaceEdit(changes)
    }
}
