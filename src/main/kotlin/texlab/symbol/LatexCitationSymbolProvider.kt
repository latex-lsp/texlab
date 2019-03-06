package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.DocumentSymbolParams
import org.eclipse.lsp4j.SymbolKind
import texlab.LatexDocument
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCitation

object LatexCitationSymbolProvider : FeatureProvider<DocumentSymbolParams, List<DocumentSymbol>> {
    override suspend fun get(request: FeatureRequest<DocumentSymbolParams>): List<DocumentSymbol> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        return request.document
                .tree
                .citations
                .map { createSymbol(it) }
    }

    private fun createSymbol(citation: LatexCitation): DocumentSymbol {
        val range = citation.name.range
        return DocumentSymbol(citation.name.text, SymbolKind.Constant, range, range)
    }
}
