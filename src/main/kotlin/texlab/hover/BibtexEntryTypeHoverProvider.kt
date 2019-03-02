package texlab.hover

import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.BibtexDocument
import texlab.contains
import texlab.metadata.BibtexEntryTypeMetadataProvider
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.BibtexEntrySyntax

object BibtexEntryTypeHoverProvider : FeatureProvider<TextDocumentPositionParams, Hover> {
    override suspend fun get(request: FeatureRequest<TextDocumentPositionParams>): List<Hover> {
        if (request.document !is BibtexDocument) {
            return emptyList()
        }

        val name = request.document.tree.root.children
                .filterIsInstance<BibtexEntrySyntax>()
                .firstOrNull { it.type.range.contains(request.params.position) }
                ?.type?.text?.substring(1)?.toLowerCase()
                ?: return emptyList()

        val metadata = BibtexEntryTypeMetadataProvider.getMetadata(name)
        val documentation = metadata?.documentation ?: return emptyList()
        return listOf(Hover(documentation))

    }
}
