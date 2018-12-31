package texlab.definition

import org.eclipse.lsp4j.Location
import texlab.BibtexDocument
import texlab.LatexDocument
import texlab.contains
import texlab.syntax.bibtex.BibtexEntrySyntax

object BibtexEntryDefinitionProvider : DefinitionProvider {
    override fun find(request: DefinitionRequest): Location? {
        if (request.document !is LatexDocument) {
            return null
        }

        val reference = request.document.tree.citations
                .firstOrNull { it.name.range.contains(request.position) } ?: return null

        for (document in request.relatedDocuments.filterIsInstance<BibtexDocument>()) {
            val definition = document.tree.root.children
                    .filterIsInstance<BibtexEntrySyntax>()
                    .firstOrNull { it.name?.text == reference.name.text }
            if (definition?.name != null) {
                return Location(document.uri.toString(), definition.name.range)
            }

        }

        return null
    }
}
