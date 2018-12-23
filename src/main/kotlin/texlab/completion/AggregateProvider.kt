package texlab.completion

import org.eclipse.lsp4j.CompletionItem

class AggregateProvider(private vararg val providers: CompletionProvider) : CompletionProvider {

    override fun getItems(request: CompletionRequest): List<CompletionItem> {
        val labels = hashSetOf<String>()
        val items = mutableListOf<CompletionItem>()
        for (provider in providers) {
            for (item in provider.getItems(request)) {
                if (labels.add(item.label)) {
                    items.add(item)
                }
            }
        }
        return items
    }
}
