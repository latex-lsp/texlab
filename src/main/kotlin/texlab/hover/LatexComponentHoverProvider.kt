package texlab.hover

import org.eclipse.lsp4j.Hover
import texlab.LatexDocument
import texlab.contains
import texlab.metadata.LatexComponentMetadataProvider

object LatexComponentHoverProvider : HoverProvider {
    override fun getHover(request: HoverRequest): Hover? {
        if (request.document !is LatexDocument) {
            return null
        }

        val name = request.document.tree.includes
                .filter { it.isUnitImport }
                .firstOrNull { it.command.range.contains(request.position) }?.path ?: return null

        val metadata = LatexComponentMetadataProvider.getMetadata(name)
        val documentation = metadata?.documentation ?: return null
        return Hover(documentation)
    }
}
