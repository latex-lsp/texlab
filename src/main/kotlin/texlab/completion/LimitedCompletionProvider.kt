package texlab.completion

import org.eclipse.lsp4j.CompletionItem

class LimitedCompletionProvider(private val provider: CompletionProvider,
                                private val limit: Int = 50) : CompletionProvider {
    override fun complete(request: CompletionRequest): List<CompletionItem> {
        return provider.complete(request)
                .take(limit)
    }
}
