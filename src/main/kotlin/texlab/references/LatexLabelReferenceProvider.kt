package texlab.references

import org.eclipse.lsp4j.Location
import texlab.LatexDocument
import texlab.contains

object LatexLabelReferenceProvider : ReferenceProvider {
    override fun getReferences(request: ReferenceRequest): List<Location>? {
        if (request.document !is LatexDocument) {
            return null
        }

        val definition = request.document.tree
                .labelDefinitions
                .firstOrNull { it.command.range.contains(request.position) }
                ?: return null

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
