package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.LatexDocument
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexUserCommandProvider : LatexCommandProvider() {
    override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return request.relatedDocuments
                .filterIsInstance<LatexDocument>()
                .flatMap { it.tree.root.descendants() }
                .filterIsInstance<LatexCommandSyntax>()
                .minus(command)
                .map { it.name.text.substring(1) }
                .distinct()
                .map { CompletionItemFactory.createCommand(it, "unknown") }
    }
}
