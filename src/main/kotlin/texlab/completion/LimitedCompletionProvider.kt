package texlab.completion

import org.eclipse.lsp4j.CompletionItem

class LimitedCompletionProvider(private val provider: CompletionProvider,
                                val limit: Int = 100) : CompletionProvider {
    override fun complete(request: CompletionRequest): List<CompletionItem> {
        return provider.complete(request)
                .take(limit)
    }
}
