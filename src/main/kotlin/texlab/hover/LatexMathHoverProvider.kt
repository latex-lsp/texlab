package texlab.hover

import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.LatexDocument
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest


abstract class LatexMathHoverProvider : FeatureProvider<TextDocumentPositionParams, Hover?> {
    override suspend fun get(request: FeatureRequest<TextDocumentPositionParams>): Hover? {
        if (request.document !is LatexDocument) {
            return null
        }

        val range = getCodeRange(request.document, request.params.position) ?: return null
        val code = request.document.tree.extract(range)
        val icon = LatexFormulaRenderer.render(code) ?: return null
        return Hover(icon)
    }

    protected abstract fun getCodeRange(document: LatexDocument, position: Position): Range?
}
