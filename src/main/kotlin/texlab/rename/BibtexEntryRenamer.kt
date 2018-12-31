package texlab.rename

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.TextEdit
import org.eclipse.lsp4j.WorkspaceEdit
import texlab.BibtexDocument
import texlab.LatexDocument
import texlab.contains
import texlab.syntax.Token
import texlab.syntax.bibtex.BibtexEntrySyntax

object BibtexEntryRenamer : Renamer {
    override fun rename(request: RenameRequest): WorkspaceEdit? {
        val token = when (request.document) {
            is BibtexDocument -> findEntry(request.document, request.position)
            is LatexDocument -> findCitation(request.document, request.position)
        } ?: return null

        val changes = mutableMapOf<String, List<TextEdit>>()
        for (document in request.relatedDocuments) {
            val edits = when (document) {
                is BibtexDocument -> {
                    document.tree.root
                            .descendants()
                            .filterIsInstance<BibtexEntrySyntax>()
                            .filter { it.name?.text == token.text }
                            .map { TextEdit(it.name!!.range, request.newName) }
                }
                is LatexDocument -> {
                    document.tree.citations
                            .filter { it.name.text == token.text }
                            .map { TextEdit(it.name.range, request.newName) }
                }
            }

            changes[document.uri.toString()] = edits
        }

        return WorkspaceEdit(changes)
    }

    private fun findEntry(document: BibtexDocument, position: Position): Token? {
        return document.tree.root.children
                .filterIsInstance<BibtexEntrySyntax>()
                .firstOrNull { it.name != null && it.name.range.contains(position) }?.name
    }

    private fun findCitation(document: LatexDocument, position: Position): Token? {
        return document.tree.citations
                .firstOrNull { it.name.range.contains(position) }?.name
    }
}
