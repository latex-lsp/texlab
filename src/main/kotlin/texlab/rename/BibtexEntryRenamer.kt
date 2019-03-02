package texlab.rename

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.RenameParams
import org.eclipse.lsp4j.TextEdit
import org.eclipse.lsp4j.WorkspaceEdit
import texlab.BibtexDocument
import texlab.LatexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.Token
import texlab.syntax.bibtex.BibtexEntrySyntax

object BibtexEntryRenamer : FeatureProvider<RenameParams, WorkspaceEdit> {
    override suspend fun get(request: FeatureRequest<RenameParams>): List<WorkspaceEdit> {
        val token = when (request.document) {
            is BibtexDocument -> findEntry(request.document, request.params.position)
            is LatexDocument -> findCitation(request.document, request.params.position)
        } ?: return emptyList()

        val changes = mutableMapOf<String, List<TextEdit>>()
        for (document in request.relatedDocuments) {
            val edits = when (document) {
                is BibtexDocument -> {
                    document.tree.root
                            .descendants()
                            .filterIsInstance<BibtexEntrySyntax>()
                            .filter { it.name?.text == token.text }
                            .map { TextEdit(it.name!!.range, request.params.newName) }
                }
                is LatexDocument -> {
                    document.tree.citations
                            .filter { it.name.text == token.text }
                            .map { TextEdit(it.name.range, request.params.newName) }
                }
            }

            changes[document.uri.toString()] = edits
        }

        return listOf(WorkspaceEdit(changes))
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
