package texlab.completion.latex.data.symbols

import org.eclipse.lsp4j.CompletionItem
import texlab.completion.AggregateCompletionProvider
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionProvider
import texlab.completion.CompletionRequest
import texlab.completion.latex.LatexArgumentProvider
import texlab.syntax.latex.LatexCommandSyntax

class LatexArgumentSymbolProvider(private val database: LatexSymbolDatabase) : CompletionProvider {
    private inner class Provider(private val symbols: List<LatexArgumentSymbol>) : LatexArgumentProvider() {
        override val commandNames: List<String> = listOf(symbols[0].command)

        override val argumentIndex: Int = symbols[0].index

        override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
            return symbols.map { createItem(it) }
        }

        private fun createItem(symbol: LatexArgumentSymbol): CompletionItem {
            return CompletionItemFactory.createArgumentSymbol(
                    symbol.argument,
                    database.resolve(symbol.imageId))
        }
    }

    val provider =
            database.index.arguments
                    .groupBy { it.command }
                    .values
                    .map { Provider(it) }
                    .let { AggregateCompletionProvider(*it.toTypedArray()) }

    override fun complete(request: CompletionRequest): List<CompletionItem> {
        return provider.complete(request)
    }
}
