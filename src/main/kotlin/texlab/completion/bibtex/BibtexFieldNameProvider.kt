package texlab.completion.bibtex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.BibtexDocument
import texlab.completion.CompletionItemFactory
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.BibtexEntrySyntax
import texlab.syntax.bibtex.BibtexFieldSyntax

object BibtexFieldNameProvider : FeatureProvider<CompletionParams, List<CompletionItem>> {
    private val ITEMS = BibtexField.values().map { CompletionItemFactory.createFieldName(it) }

    override suspend fun get(request: FeatureRequest<CompletionParams>): List<CompletionItem> {
        if (request.document !is BibtexDocument) {
            return emptyList()
        }

        val node = request.document.tree.root
                .descendants()
                .lastOrNull { it.range.contains(request.params.position) }

        val field = node is BibtexFieldSyntax && node.name.range.contains(request.params.position)
        val entry = node is BibtexEntrySyntax && !node.type.range.contains(request.params.position)
                && node.name?.range?.contains(request.params.position) != true
        return if (field || entry) {
            ITEMS
        } else {
            emptyList()
        }
    }
}
