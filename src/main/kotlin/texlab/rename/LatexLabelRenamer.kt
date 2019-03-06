package texlab.rename

import org.eclipse.lsp4j.RenameParams
import org.eclipse.lsp4j.TextEdit
import org.eclipse.lsp4j.WorkspaceEdit
import texlab.LatexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest

object LatexLabelRenamer : FeatureProvider<RenameParams, WorkspaceEdit?> {
    override suspend fun get(request: FeatureRequest<RenameParams>): WorkspaceEdit? {
        if (request.document !is LatexDocument) {
            return null
        }

        val label = request.document.tree
                .labelReferences
                .plus(request.document.tree.labelDefinitions)
                .firstOrNull { it.name.range.contains(request.params.position) }
                ?: return null

        val changes = mutableMapOf<String, List<TextEdit>>()
        for (document in request.relatedDocuments.filterIsInstance<LatexDocument>()) {
            val edits = document.tree.labelReferences
                    .plus(document.tree.labelDefinitions)
                    .filter { it.name.text == label.name.text }
                    .map { TextEdit(it.name.range, request.params.newName) }
            changes[document.uri.toString()] = edits
        }

        return WorkspaceEdit(changes)
    }
}
