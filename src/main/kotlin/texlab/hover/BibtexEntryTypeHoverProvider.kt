package texlab.hover

import org.eclipse.lsp4j.Hover
import texlab.BibtexDocument
import texlab.contains
import texlab.metadata.BibtexEntryTypeMetadataProvider
import texlab.syntax.bibtex.BibtexEntrySyntax

object BibtexEntryTypeHoverProvider : HoverProvider {
    override fun getHover(request: HoverRequest): Hover? {
        if (request.document !is BibtexDocument) {
            return null
        }

        val name = request.document.tree.root.children
                .filterIsInstance<BibtexEntrySyntax>()
                .firstOrNull { it.type.range.contains(request.position) }?.type?.text?.substring(1)?.toLowerCase()
                ?: return null

        val metadata = BibtexEntryTypeMetadataProvider.getMetadata(name)
        val documentation = metadata?.documentation ?: return null
        return Hover(documentation)
    }
}
