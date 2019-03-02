package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.DocumentSymbolParams
import org.eclipse.lsp4j.SymbolKind
import texlab.LatexDocument
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexLabel

object LatexLabelSymbolProvider : FeatureProvider<DocumentSymbolParams, DocumentSymbol> {
    override suspend fun get(request: FeatureRequest<DocumentSymbolParams>): List<DocumentSymbol> {
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
