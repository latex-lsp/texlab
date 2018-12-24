package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol

class AggregateSymbolProvider(private vararg val providers: SymbolProvider) : SymbolProvider {
    override fun getSymbols(request: SymbolRequest): List<DocumentSymbol> {
        return providers.flatMap { it.getSymbols(request) }
    }
}
