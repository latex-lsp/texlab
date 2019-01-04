package texlab.references

import org.eclipse.lsp4j.Location
import texlab.BibtexDocument
import texlab.LatexDocument
import texlab.contains
import texlab.syntax.bibtex.BibtexEntrySyntax

object BibtexEntryReferenceProvider : ReferenceProvider {
    override fun getReferences(request: ReferenceRequest): List<Location>? {
        if (request.document !is BibtexDocument) {
            return null
        }

        val definition = request.document.tree.root
                .children
                .filterIsInstance<BibtexEntrySyntax>()
                .firstOrNull { it.name != null && it.name.range.contains(request.position) }
                ?: return null

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
