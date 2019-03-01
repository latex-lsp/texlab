package texlab.hover

import kotlinx.coroutines.ObsoleteCoroutinesApi
import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.MarkupContent
import org.eclipse.lsp4j.MarkupKind
import texlab.BibtexDocument
import texlab.LatexDocument
import texlab.completion.bibtex.BibtexCitationActor
import texlab.contains
import texlab.formatting.BibtexFormatter
import texlab.syntax.bibtex.BibtexEntrySyntax

@ObsoleteCoroutinesApi
object LatexCitationHoverProvider : HoverProvider {
    override suspend fun getHover(request: HoverRequest): Hover? {
        val key = getKey(request) ?: return null

        val entry = request.relatedDocuments
                .filterIsInstance<BibtexDocument>()
                .flatMap { it.tree.root.children.filterIsInstance<BibtexEntrySyntax>() }
                .firstOrNull { it.name?.text == key }
                ?: return null

        val formatter = BibtexFormatter(insertSpaces = true, tabSize = 4, lineLength = -1)
        return Hover(MarkupContent().apply {
            kind = MarkupKind.MARKDOWN
            value = BibtexCitationActor.cite(formatter.format(entry))
        })
    }

    private fun getKey(request: HoverRequest): String? {
        return when (request.document) {
            is LatexDocument -> {
                request.document.tree.citations
                        .firstOrNull { it.command.range.contains(request.position) }
                        ?.name?.text
            }
            is BibtexDocument -> {
                request.document.tree.root.children
                        .filterIsInstance<BibtexEntrySyntax>()
                        .firstOrNull { it.name != null && it.name.range.contains(request.position) }
                        ?.name?.text
            }
        }
    }
}
