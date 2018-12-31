package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexKernelCommandProvider : LatexCommandProvider() {
    val ITEMS = KernelPrimitives.COMMANDS
            .map { CompletionItemFactory.createCommand(it, null) }

    override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return ITEMS
    }
}
