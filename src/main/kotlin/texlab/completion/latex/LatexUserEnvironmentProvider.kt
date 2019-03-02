package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.LatexDocument
import texlab.completion.CompletionItemFactory
import texlab.contains
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexUserEnvironmentProvider : LatexEnvironmentProvider() {
    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: LatexCommandSyntax): List<CompletionItem> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val current = request.document
                .tree
                .environments
                .firstOrNull {
                    it.beginNameRange.contains(request.params.position) ||
                            it.endNameRange.contains(request.params.position)
                }
        val excluded = if (current == null) {
            emptyList()
        } else {
            listOf(current)
        }

        return request.relatedDocuments
                .filterIsInstance<LatexDocument>()
                .flatMap { it.tree.environments }
                .minus(excluded)
                .flatMap { listOf(it.beginName, it.endName) }
                .filter { it != "" }
                .distinct()
                .map { CompletionItemFactory.createEnvironment(it, "unknown") }
    }
}
