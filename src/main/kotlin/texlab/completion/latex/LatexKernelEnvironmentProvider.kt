package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.CompletionItemFactory
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexKernelEnvironmentProvider : LatexEnvironmentProvider() {
    private val items = KernelPrimitives
            .ENVIRONMENTS
            .map { CompletionItemFactory.createEnvironment(it, null) }

    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: LatexCommandSyntax): List<CompletionItem> {
        return items
    }
}
