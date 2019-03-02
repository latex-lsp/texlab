package texlab.completion.bibtex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.latex.LatexKernelCommandProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.BibtexCommandSyntax

object BibtexKernelCommandProvider : BibtexCommandProvider() {
    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: BibtexCommandSyntax): List<CompletionItem> {
        return LatexKernelCommandProvider.ITEMS
    }
}
