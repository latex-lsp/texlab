package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol

interface SymbolProvider {
    fun getSymbols(request: SymbolRequest): List<DocumentSymbol>
}

