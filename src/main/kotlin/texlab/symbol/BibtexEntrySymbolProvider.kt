package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.SymbolKind
import texlab.BibtexDocument
import texlab.syntax.bibtex.BibtexEntrySyntax

object BibtexEntrySymbolProvider : SymbolProvider {
    override fun getSymbols(request: SymbolRequest): List<DocumentSymbol> {
        if (request.document !is BibtexDocument) {
            return emptyList()
        }

        return request.document
                .tree
                .root
                .descendants()
                .filterIsInstance<BibtexEntrySyntax>()
                .mapNotNull { createSymbol(it) }
    }

    private fun createSymbol(entry: BibtexEntrySyntax): DocumentSymbol? {
        if (entry.name == null) {
            return null
        }

        val range = entry.name.range
        return DocumentSymbol(entry.name.text, SymbolKind.Field, range, range)
    }
}
