package texlab.completion

import org.eclipse.lsp4j.CompletionItem

class AggregateCompletionProvider(private vararg val providers: CompletionProvider) : CompletionProvider {
    override fun complete(request: CompletionRequest): List<CompletionItem> {
        val labels = hashSetOf<String>()
        val items = mutableListOf<CompletionItem>()
        for (provider in providers) {
            for (item in provider.complete(request)) {
                if (labels.add(item.label)) {
                    items.add(item)
                }
            }
        }
        return items
    }
}
