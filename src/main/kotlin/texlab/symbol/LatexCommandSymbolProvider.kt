package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.SymbolKind
import texlab.LatexDocument
import texlab.syntax.latex.LatexCommandSyntax

object LatexCommandSymbolProvider : SymbolProvider {
    override fun getSymbols(request: SymbolRequest): List<DocumentSymbol> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        return request.document.tree.root.descendants()
                .filterIsInstance<LatexCommandSyntax>()
                .map { DocumentSymbol(it.name.text, SymbolKind.Method, it.name.range, it.name.range) }
    }
}
