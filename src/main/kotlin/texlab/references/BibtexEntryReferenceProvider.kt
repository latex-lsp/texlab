package texlab.references

import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.ReferenceParams
import texlab.BibtexDocument
import texlab.LatexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.BibtexEntrySyntax

object BibtexEntryReferenceProvider : FeatureProvider<ReferenceParams, List<Location>> {
    override suspend fun get(request: FeatureRequest<ReferenceParams>): List<Location> {
        if (request.document !is BibtexDocument) {
            return emptyList()
        }

        val definition = request.document.tree.root
                .children
                .filterIsInstance<BibtexEntrySyntax>()
                .firstOrNull { it.name != null && it.name.range.contains(request.params.position) }
                ?: return emptyList()

        val references = mutableListOf<Location>()
        for (document in request.relatedDocuments.filterIsInstance<LatexDocument>()) {
            document.tree.citations
                    .filter { it.name.text == definition.name!!.text }
                    .map { Location(document.uri.toString(), it.command.range) }
                    .also { references.addAll(it) }
        }
        return references
    }
}
