package texlab.completion

import org.eclipse.lsp4j.CompletionItem

interface CompletionProvider {

    fun getItems(request: CompletionRequest): Sequence<CompletionItem>
}
