package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.LatexDocument
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.latex.LatexCommandSyntax

class LatexUserEnvironmentProvider : LatexEnvironmentProvider() {

    override fun getItems(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return request.relatedDocuments
                .filterIsInstance<LatexDocument>()
                .flatMap { it.tree.environments }
                .flatMap { listOf(it.beginName, it.endName) }
                .filter { it != "" }
                .distinct()
                .map { CompletionItemFactory.createEnvironment(it, "unknown") }
    }
}
