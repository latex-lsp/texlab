package texlab.completion

import org.eclipse.lsp4j.CompletionItem

class LimitedCompletionProvider(private val provider: CompletionProvider,
                                val limit: Int = 100) : CompletionProvider {
    override fun getItems(request: CompletionRequest): List<CompletionItem> {
        return provider.getItems(request)
                .take(limit)
    }
}
