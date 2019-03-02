package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.CompletionItemFactory
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexBeginCommandProvider : LatexCommandProvider() {
    private val snippet = CompletionItemFactory.createSnippet("begin", null, "begin{$1}\n\t$0\n\\end{$1}")

    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: LatexCommandSyntax): List<CompletionItem> {
        return listOf(snippet)
    }
}
