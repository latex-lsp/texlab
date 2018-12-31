package texlab.completion.bibtex

import org.eclipse.lsp4j.CompletionItem
import texlab.BibtexDocument
import texlab.completion.CompletionProvider
import texlab.completion.CompletionRequest
import texlab.contains
import texlab.syntax.bibtex.BibtexCommandSyntax

abstract class BibtexCommandProvider : CompletionProvider {
    override fun complete(request: CompletionRequest): List<CompletionItem> {
        if (request.document !is BibtexDocument) {
            return emptyList()
        }

        val command = request.document.tree.root
                .descendants()
                .lastOrNull { it.range.contains(request.position) }

        return if (command is BibtexCommandSyntax) {
            complete(request, command)
        } else {
            emptyList()
        }
    }

    protected abstract fun complete(request: CompletionRequest, command: BibtexCommandSyntax): List<CompletionItem>
}
