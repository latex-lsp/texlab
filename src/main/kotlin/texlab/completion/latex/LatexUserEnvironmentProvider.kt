package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.LatexDocument
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.contains
import texlab.syntax.latex.LatexCommandSyntax

class LatexUserEnvironmentProvider : LatexEnvironmentProvider() {

    override fun getItems(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val environment = request.document
                .tree
                .environments
                .firstOrNull {
                    it.beginNameRange.contains(request.position) ||
                            it.endNameRange.contains(request.position)
                } ?: return emptyList()

        return request.relatedDocuments
                .filterIsInstance<LatexDocument>()
                .flatMap { it.tree.environments }
                .minus(environment)
                .flatMap { listOf(it.beginName, it.endName) }
                .filter { it != "" }
                .distinct()
                .map { CompletionItemFactory.createEnvironment(it, "unknown") }
    }
}
