package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.DocumentSymbolParams
import org.eclipse.lsp4j.SymbolKind
import texlab.LatexDocument
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexCommandSymbolProvider : FeatureProvider<DocumentSymbolParams, List<DocumentSymbol>> {
    override suspend fun get(request: FeatureRequest<DocumentSymbolParams>): List<DocumentSymbol> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        return request.document.tree.root.descendants()
                .filterIsInstance<LatexCommandSyntax>()
                .map { DocumentSymbol(it.name.text, SymbolKind.Method, it.name.range, it.name.range) }
    }
}
