package texlab.hover

import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.MarkupContent
import org.eclipse.lsp4j.MarkupKind
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.BibtexDocument
import texlab.completion.bibtex.BibtexField
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.BibtexFieldSyntax

object BibtexFieldHoverProvider : FeatureProvider<TextDocumentPositionParams, Hover?> {
    override suspend fun get(request: FeatureRequest<TextDocumentPositionParams>): Hover? {
        if (request.document !is BibtexDocument) {
            return null
        }

        val fieldNode = request.document.tree.root.descendants()
                .filterIsInstance<BibtexFieldSyntax>()
                .firstOrNull { it.name.range.contains(request.params.position) }
                ?: return null

        val fieldName = BibtexField.parse(fieldNode.name.text) ?: return null
        val markup = MarkupContent().apply {
            kind = MarkupKind.MARKDOWN
            value = fieldName.documentation()
        }
        return Hover(markup)
    }
}
