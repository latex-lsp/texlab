package texlab.definition

import org.eclipse.lsp4j.Location
import texlab.LatexDocument
import texlab.contains

object LatexLabelDefinitionProvider : DefinitionProvider {
    override fun find(request: DefinitionRequest): Location? {
        if (request.document !is LatexDocument) {
            return null
        }

        val reference = request.document.tree.labelReferences
                .plus(request.document.tree.labelDefinitions)
                .firstOrNull { it.name.range.contains(request.position) } ?: return null

        for (document in request.relatedDocuments.filterIsInstance<LatexDocument>()) {
            val definition = document.tree.labelDefinitions
                    .firstOrNull { it.name.text == reference.name.text }
            if (definition != null) {
                return Location(document.uri.toString(), definition.command.range)
            }
        }
        return null
    }
}
