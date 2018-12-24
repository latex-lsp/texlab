package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.SymbolKind
import texlab.LatexDocument
import texlab.syntax.latex.LatexLabel

object LatexLabelSymbolProvider : SymbolProvider {
    override fun getSymbols(request: SymbolRequest): List<DocumentSymbol> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        return request.document
                .tree
                .labelReferences
                .plus(request.document.tree.labelDefinitions)
                .map { createSymbol(it) }
    }

    private fun createSymbol(label: LatexLabel): DocumentSymbol {
        val range = label.name.range
        return DocumentSymbol(label.name.text, SymbolKind.Field, range, range)
    }
}
