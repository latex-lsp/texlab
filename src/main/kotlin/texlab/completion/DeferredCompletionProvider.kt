package texlab.completion

import kotlinx.coroutines.Deferred
import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.CompletionItem

class DeferredCompletionProvider<T>(private val providerFactory: (source: T) -> CompletionProvider,
                                    private val source: Deferred<T>) : CompletionProvider {
    private var provider: CompletionProvider? = null

    override fun complete(request: CompletionRequest): List<CompletionItem> = runBlocking {
        if (provider == null && source.isCompleted) {
            provider = providerFactory(source.await())
        }

        provider?.complete(request) ?: emptyList()
    }
}
