package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexBeginCommandProvider : LatexCommandProvider() {
    private val snippet = CompletionItemFactory.createSnippet("begin", null, "begin{$1}\n\t$0\n\\end{$1}")

    override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return listOf(snippet)
    }
}
