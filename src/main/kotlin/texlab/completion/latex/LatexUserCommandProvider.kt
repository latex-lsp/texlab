package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.LatexDocument
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.latex.LatexCommandSyntax

class LatexUserCommandProvider : LatexCommandProvider() {

    override fun getItems(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return request.relatedDocuments
                .filterIsInstance<LatexDocument>()
                .flatMap { it.tree.root.descendants() }
                .filterIsInstance<LatexCommandSyntax>()
                .map { it.name.text.substring(1) }
                .filter { it != command.name.text }
                .distinct()
                .map { CompletionItemFactory.createCommand(it, "unknown") }
    }
}
