package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.LatexDocument
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.contains
import texlab.syntax.latex.LatexCommandSyntax
import texlab.syntax.latex.LatexEnvironment

object LatexUserEnvironmentProvider : LatexEnvironmentProvider() {
    override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val current = request.document
                .tree
                .environments
                .firstOrNull {
                    it.beginNameRange.contains(request.position) ||
                            it.endNameRange.contains(request.position)
                }
        val excluded = if (current == null) {
            emptyList<LatexEnvironment>()
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
