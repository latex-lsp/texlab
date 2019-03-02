package texlab.references

import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.ReferenceParams
import texlab.LatexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest

object LatexLabelReferenceProvider : FeatureProvider<ReferenceParams, Location> {
    override suspend fun get(request: FeatureRequest<ReferenceParams>): List<Location> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val definition = request.document.tree
                .labelDefinitions
                .firstOrNull { it.command.range.contains(request.params.position) }
                ?: return emptyList()

        val references = mutableListOf<Location>()
        for (document in request.relatedDocuments.filterIsInstance<LatexDocument>()) {
            document.tree.labelReferences
                    .filter { it.name.text == definition.name.text }
                    .map { Location(document.uri.toString(), it.command.range) }
                    .also { references.addAll(it) }
        }
        return references

    }
}
