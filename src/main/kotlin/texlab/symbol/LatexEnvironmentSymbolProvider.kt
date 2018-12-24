package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.Range
import org.eclipse.lsp4j.SymbolKind
import texlab.LatexDocument

object LatexEnvironmentSymbolProvider : SymbolProvider {
    override fun getSymbols(request: SymbolRequest): List<DocumentSymbol> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val symbols = mutableListOf<DocumentSymbol>()
        for (environment in request.document.tree.environments) {
            symbols.add(createSymbol(environment.beginName, environment.beginNameRange))
            symbols.add(createSymbol(environment.endName, environment.endNameRange))
        }
        return symbols
    }

    private fun createSymbol(name: String, range: Range): DocumentSymbol {
        return DocumentSymbol(name, SymbolKind.EnumMember, range, range)
    }
}
