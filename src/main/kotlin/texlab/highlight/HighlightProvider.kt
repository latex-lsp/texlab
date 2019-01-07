package texlab.highlight

import org.eclipse.lsp4j.DocumentHighlight

interface HighlightProvider {
    fun getHighlights(request: HighlightRequest): List<DocumentHighlight>?
}
