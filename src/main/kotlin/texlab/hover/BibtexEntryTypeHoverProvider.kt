package texlab.hover

import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.BibtexDocument
import texlab.contains
import texlab.metadata.BibtexEntryTypeMetadataProvider
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.BibtexEntrySyntax

object BibtexEntryTypeHoverProvider : FeatureProvider<TextDocumentPositionParams, Hover?> {
    override suspend fun get(request: FeatureRequest<TextDocumentPositionParams>): Hover? {
        if (request.document !is BibtexDocument) {
            return null
        }

        val name = request.document.tree.root.children
                .filterIsInstance<BibtexEntrySyntax>()
                .firstOrNull { it.type.range.contains(request.params.position) }
                ?.type?.text?.substring(1)?.toLowerCase()
                ?: return null

        val metadata = BibtexEntryTypeMetadataProvider.getMetadata(name)
        val documentation = metadata?.documentation ?: return null
        return Hover(documentation)
    }
}
