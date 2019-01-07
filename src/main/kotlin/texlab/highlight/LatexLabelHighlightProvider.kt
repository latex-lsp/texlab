package texlab.highlight

import org.eclipse.lsp4j.DocumentHighlight
import org.eclipse.lsp4j.DocumentHighlightKind
import texlab.LatexDocument
import texlab.contains
import texlab.syntax.latex.LatexLabel

object LatexLabelHighlightProvider : HighlightProvider {
    override fun getHighlights(request: HighlightRequest): List<DocumentHighlight>? {
        if (request.document !is LatexDocument) {
            return null
        }

        val label = request.document.tree.labelDefinitions
                .plus(request.document.tree.labelReferences)
                .firstOrNull { it.name.range.contains(request.position) }
                ?: return null

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
