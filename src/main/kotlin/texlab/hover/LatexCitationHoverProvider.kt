package texlab.hover

import kotlinx.coroutines.ObsoleteCoroutinesApi
import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.MarkupContent
import org.eclipse.lsp4j.MarkupKind
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.BibtexDocument
import texlab.LatexDocument
import texlab.completion.bibtex.BibtexCitationActor
import texlab.contains
import texlab.formatting.BibtexFormatter
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.BibtexEntrySyntax

@ObsoleteCoroutinesApi
object LatexCitationHoverProvider : FeatureProvider<TextDocumentPositionParams, Hover?> {
    override suspend fun get(request: FeatureRequest<TextDocumentPositionParams>): Hover? {
        val key = getKey(request) ?: return null
        val entry = request.relatedDocuments
                .filterIsInstance<BibtexDocument>()
                .flatMap { it.tree.root.children.filterIsInstance<BibtexEntrySyntax>() }
                .firstOrNull { it.name?.text == key }
                ?: return null

        val formatter = BibtexFormatter(insertSpaces = true, tabSize = 4, lineLength = -1)
        val markup = MarkupContent().apply {
            kind = MarkupKind.MARKDOWN
            value = BibtexCitationActor.cite(formatter.format(entry))
        }
        return Hover(markup)
    }

    private fun getKey(request: FeatureRequest<TextDocumentPositionParams>): String? {
        if (request.params.position == null) {
            return null
        }

        return when (request.document) {
            is LatexDocument -> {
                request.document.tree.citations
                        .firstOrNull { it.command.range.contains(request.params.position) }
                        ?.name?.text
            }
            is BibtexDocument -> {
                request.document.tree.root.children
                        .filterIsInstance<BibtexEntrySyntax>()
                        .firstOrNull { it.name != null && it.name.range.contains(request.params.position) }
                        ?.name?.text
            }
        }
    }
}
