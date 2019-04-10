package texlab.completion.latex.data.symbols

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.CompletionItemFactory
import texlab.completion.latex.LatexArgumentProvider
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexArgumentSymbolProvider : FeatureProvider<CompletionParams, List<CompletionItem>> {
    private class Provider(symbol: LatexArgumentSymbol) : LatexArgumentProvider() {
        override val commandNames: List<String> = listOf("\\" + symbol.command)

        override val argumentIndex: Int = symbol.index

        val items = symbol.arguments.map { CompletionItemFactory.createArgumentSymbol(it.argument, it.image) }

        override fun complete(request: FeatureRequest<CompletionParams>,
                              command: LatexCommandSyntax): List<CompletionItem> {
            return items
        }
    }

    val provider = LatexSymbolDatabase.INSTANCE.arguments
            .map { Provider(it) }
            .let { FeatureProvider.concat(*it.toTypedArray()) }

    override suspend fun get(request: FeatureRequest<CompletionParams>): List<CompletionItem> {
        return provider.get(request)
    }
}
