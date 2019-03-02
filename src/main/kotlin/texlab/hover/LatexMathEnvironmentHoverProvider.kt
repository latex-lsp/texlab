package texlab.hover

import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.MarkupContent
import org.eclipse.lsp4j.MarkupKind
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.LatexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest


object LatexMathEnvironmentHoverProvider : FeatureProvider<TextDocumentPositionParams, Hover> {
    override suspend fun get(request: FeatureRequest<TextDocumentPositionParams>): List<Hover> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val environment = request.document.tree
                .environments
                .firstOrNull { it.range.contains(request.params.position) }
                ?: return emptyList()

        val name = environment.beginName.replace("*", "")
        if (!LatexFormulaRenderer.ENVIRONMENTS.contains(name)) {
            return emptyList()
        }

        val source = request.document.tree.extract(environment.range)
        val base64 = LatexFormulaRenderer.renderBase64(source) ?: return emptyList()
        return listOf(Hover(MarkupContent().apply {
            kind = MarkupKind.MARKDOWN
            value = "![formula](data:image/png;base64,$base64)"
        }))
    }
}
