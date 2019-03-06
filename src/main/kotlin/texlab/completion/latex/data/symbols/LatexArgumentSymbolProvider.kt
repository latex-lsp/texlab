package texlab.completion.latex.data.symbols

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.CompletionItemFactory
import texlab.completion.latex.LatexArgumentProvider
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

class LatexArgumentSymbolProvider(private val database: LatexSymbolDatabase)
    : FeatureProvider<CompletionParams, List<CompletionItem>> {

    private inner class Provider(private val symbols: List<LatexArgumentSymbol>) : LatexArgumentProvider() {
        override val commandNames: List<String> = listOf(symbols[0].command)

        override val argumentIndex: Int = symbols[0].index

        override fun complete(request: FeatureRequest<CompletionParams>,
                              command: LatexCommandSyntax): List<CompletionItem> {
            return symbols.map { createItem(it) }
        }

        private fun createItem(symbol: LatexArgumentSymbol): CompletionItem {
            return CompletionItemFactory.createArgumentSymbol(
                    symbol.argument,
                    database.resolve(symbol.imageId))
        }
    }

    val provider = database.index.arguments
            .groupBy { it.command }
            .values
            .map { Provider(it) }
            .let { FeatureProvider.concat(*it.toTypedArray()) }

    override suspend fun get(request: FeatureRequest<CompletionParams>): List<CompletionItem> {
        return provider.get(request)
    }
}
