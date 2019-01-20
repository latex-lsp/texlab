package texlab.completion.bibtex

import org.eclipse.lsp4j.CompletionItem
import texlab.BibtexDocument
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionProvider
import texlab.completion.CompletionRequest
import texlab.contains
import texlab.syntax.bibtex.BibtexEntrySyntax
import texlab.syntax.bibtex.BibtexFieldSyntax

object BibtexFieldNameProvider : CompletionProvider {
    private val ITEMS = BibtexField.values().map { CompletionItemFactory.createFieldName(it) }

    override fun complete(request: CompletionRequest): List<CompletionItem> {
        if (request.document !is BibtexDocument) {
            return emptyList()
        }

        val node = request.document.tree.root
                .descendants()
                .lastOrNull { it.range.contains(request.position) }

        val field = node is BibtexFieldSyntax && node.name.range.contains(request.position)
        val entry = node is BibtexEntrySyntax && !node.type.range.contains(request.position)
                && node.name?.range?.contains(request.position) != true
        return if (field || entry) {
            ITEMS
        } else {
            emptyList()
        }
    }
}
