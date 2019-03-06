package texlab.link

import org.eclipse.lsp4j.DocumentLink
import org.eclipse.lsp4j.DocumentLinkParams
import texlab.LatexDocument
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexInclude

object LatexIncludeLinkProvider : FeatureProvider<DocumentLinkParams, List<DocumentLink>> {
    override suspend fun get(request: FeatureRequest<DocumentLinkParams>): List<DocumentLink> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        return request.document
                .tree
                .includes
                .mapNotNull { resolve(request, it) }
    }

    private fun resolve(request: FeatureRequest<DocumentLinkParams>,
                        include: LatexInclude): DocumentLink? {
        val range = include.command.args[0].children[0].range
        val target = request.workspace.resolveDocument(request.uri, include.path) ?: return null
        return DocumentLink(range, target.uri.toString())
    }
}
