package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexKernelEnvironmentProvider : LatexEnvironmentProvider() {

    private val items = KernelPrimitives
            .ENVIRONMENTS
            .map { CompletionItemFactory.createEnvironment(it, null) }

    override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return items
    }
}
