package texlab.highlight

import org.eclipse.lsp4j.DocumentHighlight
import org.eclipse.lsp4j.DocumentHighlightKind
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.LatexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexLabel

object LatexLabelHighlightProvider : FeatureProvider<TextDocumentPositionParams, DocumentHighlight> {
    override suspend fun get(request: FeatureRequest<TextDocumentPositionParams>): List<DocumentHighlight> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val label = request.document.tree.labelDefinitions
                .plus(request.document.tree.labelReferences)
                .firstOrNull { it.name.range.contains(request.params.position) }
                ?: return emptyList()

        return request.document.tree.labelDefinitions
                .plus(request.document.tree.labelReferences)
                .filter { it.name.text == label.name.text }
                .map { DocumentHighlight(it.name.range, getHighlightKind(it)) }
    }

    private fun getHighlightKind(label: LatexLabel): DocumentHighlightKind {
        return if (label.command.name.text == "\\label") {
            DocumentHighlightKind.Write
        } else {
            DocumentHighlightKind.Read
        }
    }
}
