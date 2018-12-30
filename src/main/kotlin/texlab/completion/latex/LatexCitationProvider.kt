package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.BibtexDocument
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.bibtex.BibtexEntrySyntax
import texlab.syntax.latex.LatexCitation
import texlab.syntax.latex.LatexCommandSyntax

object LatexCitationProvider : LatexArgumentProvider() {
    override val commandNames: List<String> = LatexCitation.COMMAND_NAMES

    override val argumentIndex: Int = 0

    override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return request.relatedDocuments
                .filterIsInstance<BibtexDocument>()
                .flatMap { it.tree.root.children }
                .filterIsInstance<BibtexEntrySyntax>()
                .mapNotNull { it.name?.text }
                .map { CompletionItemFactory.createCitation(it) }
    }
}
