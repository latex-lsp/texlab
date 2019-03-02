package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.DocumentSymbolParams
import org.eclipse.lsp4j.Range
import org.eclipse.lsp4j.SymbolKind
import texlab.LatexDocument
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest

object LatexEnvironmentSymbolProvider : FeatureProvider<DocumentSymbolParams, DocumentSymbol> {
    override suspend fun get(request: FeatureRequest<DocumentSymbolParams>): List<DocumentSymbol> {
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
