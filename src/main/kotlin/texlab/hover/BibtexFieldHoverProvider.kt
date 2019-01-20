package texlab.hover

import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.MarkupContent
import org.eclipse.lsp4j.MarkupKind
import texlab.BibtexDocument
import texlab.completion.bibtex.BibtexField
import texlab.contains
import texlab.syntax.bibtex.BibtexFieldSyntax

object BibtexFieldHoverProvider : HoverProvider {
    override fun getHover(request: HoverRequest): Hover? {
        if (request.document !is BibtexDocument) {
            return null
        }

        val fieldNode = request.document.tree.root.descendants()
                .filterIsInstance<BibtexFieldSyntax>()
                .firstOrNull { it.name.range.contains(request.position) }
                ?: return null

        val fieldName = BibtexField.parse(fieldNode.name.text) ?: return null
        val markup = MarkupContent().apply {
            kind = MarkupKind.MARKDOWN
            value = fieldName.documentation()
        }
        return Hover(markup)
    }
}
