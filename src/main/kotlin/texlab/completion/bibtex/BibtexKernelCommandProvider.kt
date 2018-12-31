package texlab.completion.bibtex

import org.eclipse.lsp4j.CompletionItem
import texlab.completion.CompletionRequest
import texlab.completion.latex.LatexKernelCommandProvider
import texlab.syntax.bibtex.BibtexCommandSyntax

object BibtexKernelCommandProvider : BibtexCommandProvider() {
    override fun complete(request: CompletionRequest, command: BibtexCommandSyntax): List<CompletionItem> {
        return LatexKernelCommandProvider.ITEMS
    }
}
