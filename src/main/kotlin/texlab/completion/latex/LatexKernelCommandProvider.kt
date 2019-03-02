package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.CompletionItemFactory
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexKernelCommandProvider : LatexCommandProvider() {
    val ITEMS = KernelPrimitives.COMMANDS
            .map { CompletionItemFactory.createCommand(it, null) }

    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: LatexCommandSyntax): List<CompletionItem> {
        return ITEMS
    }
}
