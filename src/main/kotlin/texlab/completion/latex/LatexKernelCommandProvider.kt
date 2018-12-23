package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.latex.LatexCommandSyntax

class LatexKernelCommandProvider : LatexCommandProvider() {

    private val items = KernelPrimitives
            .COMMANDS
            .map { CompletionItemFactory.createCommand(it, null) }

    override fun getItems(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return items
    }
}
