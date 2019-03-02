package texlab.definition

import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.LatexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest

object LatexLabelDefinitionProvider : FeatureProvider<TextDocumentPositionParams, Location> {
    override suspend fun get(request: FeatureRequest<TextDocumentPositionParams>): List<Location> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val reference = request.document.tree.labelReferences
                .plus(request.document.tree.labelDefinitions)
                .firstOrNull { it.name.range.contains(request.params.position) } ?: return emptyList()

        for (document in request.relatedDocuments.filterIsInstance<LatexDocument>()) {
            val definition = document.tree.labelDefinitions
                    .firstOrNull { it.name.text == reference.name.text }
            if (definition != null) {
                return listOf(Location(document.uri.toString(), definition.name.range))
            }
        }
        return emptyList()
    }
}
