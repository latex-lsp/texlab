package texlab.link

import org.eclipse.lsp4j.DocumentLink
import texlab.LatexDocument
import texlab.syntax.latex.LatexInclude

object LatexIncludeLinkProvider : LinkProvider {
    override fun getLinks(request: LinkRequest): List<DocumentLink> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        return request.document
                .tree
                .includes
                .mapNotNull { resolve(request, it) }
    }

    private fun resolve(request: LinkRequest, include: LatexInclude): DocumentLink? {
        val range = include.command.args[0].children[0].range
        val target = request.workspace.resolve(request.uri, include.path) ?: return null
        return DocumentLink(range, target.uri.toString())
    }
}
