package texlab.highlight

import org.eclipse.lsp4j.DocumentHighlight

class AggregateHighlightProvider(private vararg val providers: HighlightProvider) : HighlightProvider {
    override fun getHighlights(request: HighlightRequest): List<DocumentHighlight>? {
        for (provider in providers) {
            val highlights = provider.getHighlights(request)
            if (highlights != null) {
                return highlights
            }
        }
        return null
    }
}
