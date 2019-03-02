package texlab.rename

import org.eclipse.lsp4j.RenameParams
import org.eclipse.lsp4j.TextEdit
import org.eclipse.lsp4j.WorkspaceEdit
import texlab.LatexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexCommandRenamer : FeatureProvider<RenameParams, WorkspaceEdit> {
    override suspend fun get(request: FeatureRequest<RenameParams>): List<WorkspaceEdit> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val command = request.document.tree.root
                .descendants()
                .filterIsInstance<LatexCommandSyntax>()
                .firstOrNull { it.name.range.contains(request.params.position) } ?: return emptyList()

        val changes = mutableMapOf<String, List<TextEdit>>()
        for (document in request.relatedDocuments.filterIsInstance<LatexDocument>()) {
            val edits = document.tree.root.descendants()
                    .filterIsInstance<LatexCommandSyntax>()
                    .filter { it.name.text == command.name.text }
                    .map { TextEdit(it.name.range, "\\" + request.params.newName) }
            changes[document.uri.toString()] = edits
        }

        return listOf(WorkspaceEdit(changes))
    }
}
