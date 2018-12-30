package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.SymbolKind
import texlab.LatexDocument
import texlab.syntax.latex.LatexCitation

object LatexCitationSymbolProvider : SymbolProvider {
    override fun getSymbols(request: SymbolRequest): List<DocumentSymbol> {
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
