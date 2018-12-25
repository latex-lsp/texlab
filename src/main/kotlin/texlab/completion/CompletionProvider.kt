package texlab.completion

import org.eclipse.lsp4j.CompletionItem

interface CompletionProvider {
    fun complete(request: CompletionRequest): List<CompletionItem>
}
