package texlab.rename

import org.eclipse.lsp4j.TextEdit
import org.eclipse.lsp4j.WorkspaceEdit
import texlab.LatexDocument
import texlab.contains
import texlab.syntax.latex.LatexCommandSyntax

object LatexCommandRenamer : Renamer {
    override fun rename(request: RenameRequest): WorkspaceEdit? {
        if (request.document !is LatexDocument) {
            return null
        }

        val command = request.document.tree.root
                .descendants()
                .filterIsInstance<LatexCommandSyntax>()
                .firstOrNull { it.name.range.contains(request.position) } ?: return null

        val changes = mutableMapOf<String, List<TextEdit>>()
        for (document in request.relatedDocuments.filterIsInstance<LatexDocument>()) {
            val edits = document.tree.root.descendants()
                    .filterIsInstance<LatexCommandSyntax>()
                    .filter { it.name.text == command.name.text }
                    .map { TextEdit(it.name.range, "\\" + request.newName) }
            changes[document.uri.toString()] = edits
        }
        return WorkspaceEdit(changes)
    }
}
