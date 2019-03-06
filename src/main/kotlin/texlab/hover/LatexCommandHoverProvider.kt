package texlab.hover

import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.MarkupContent
import org.eclipse.lsp4j.MarkupKind
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.LatexDocument
import texlab.completion.latex.data.LatexComponentSource
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

class LatexCommandHoverProvider(private val database: LatexComponentSource) :
        FeatureProvider<TextDocumentPositionParams, List<Hover>> {
    override suspend fun get(request: FeatureRequest<TextDocumentPositionParams>): List<Hover> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val command = request.document.tree.root
                .descendants()
                .filterIsInstance<LatexCommandSyntax>()
                .firstOrNull { it.name.range.contains(request.params.position) }
                ?: return emptyList()

        val components = database.getRelatedComponents(request.relatedDocuments)
                .filter { it.commands.contains(command.name.text.substring(1)) }
                .flatMap { it.fileNames }

        val separator = System.lineSeparator().repeat(2)
        return listOf(Hover(MarkupContent().apply {
            kind = MarkupKind.MARKDOWN
            value = components.joinToString(separator)
        }))
    }
}
