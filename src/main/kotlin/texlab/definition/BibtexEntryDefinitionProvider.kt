package texlab.definition

import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.BibtexDocument
import texlab.LatexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.BibtexEntrySyntax

object BibtexEntryDefinitionProvider : FeatureProvider<TextDocumentPositionParams, List<Location>> {
    override suspend fun get(request: FeatureRequest<TextDocumentPositionParams>): List<Location> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val reference = request.document.tree.citations
                .firstOrNull { it.name.range.contains(request.params.position) } ?: return emptyList()

        for (document in request.relatedDocuments.filterIsInstance<BibtexDocument>()) {
            val definition = document.tree.root.children
                    .filterIsInstance<BibtexEntrySyntax>()
                    .firstOrNull { it.name?.text == reference.name.text }
            if (definition?.name != null) {
                return listOf(Location(document.uri.toString(), definition.name.range))
            }

        }

        return emptyList()
    }
}
