package texlab.hover

import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.LatexDocument
import texlab.contains
import texlab.metadata.LatexComponentMetadataProvider
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest

object LatexComponentHoverProvider : FeatureProvider<TextDocumentPositionParams, Hover> {
    override suspend fun get(request: FeatureRequest<TextDocumentPositionParams>): List<Hover> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val name = request.document.tree.includes
                .filter { it.isUnitImport }
                .firstOrNull { it.command.range.contains(request.params.position) }
                ?.path ?: return emptyList()

        val metadata = LatexComponentMetadataProvider.getMetadata(name)
        val documentation = metadata?.documentation ?: return emptyList()
        return listOf(Hover(documentation))
    }
}
