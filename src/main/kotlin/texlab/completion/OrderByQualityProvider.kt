package texlab.completion

import org.eclipse.lsp4j.CompletionItem
import texlab.BibtexDocument
import texlab.LatexDocument
import texlab.contains
import texlab.syntax.latex.LatexCommandSyntax
import texlab.syntax.latex.LatexDocumentSyntax
import texlab.syntax.latex.LatexGroupSyntax
import texlab.syntax.latex.LatexTextSyntax

class OrderByQualityProvider(private val provider: CompletionProvider) : CompletionProvider {

    override fun complete(request: CompletionRequest): List<CompletionItem> {
        val name = getName(request)
        return if (name == null) {
            listOf()
        } else {
            provider.complete(request)
                    .sortedByDescending { getQuality(it.label, name) }
        }
    }

    private fun getName(request: CompletionRequest): String? {
        if (request.document is LatexDocument) {
            val node = request.document
                    .tree
                    .root
                    .descendants()
                    .lastOrNull { it.range.contains(request.position) }

            return when (node) {
                is LatexGroupSyntax -> ""
                is LatexCommandSyntax -> node.name.text.substring(1)
                is LatexTextSyntax -> node.words[0].text
                is LatexDocumentSyntax -> null
                null -> null
            }
        } else {
            request.document as BibtexDocument
            return null
        }
    }

    private fun getQuality(label: String, query: String): Int {
        if (label == query) {
            return 7
        }

        if (label.equals(query, ignoreCase = true)) {
            return 6
        }

        if (label.startsWith(query)) {
            return 5
        }

        if (label.toLowerCase().startsWith(query.toLowerCase())) {
            return 4
        }

        if (label.contains(query)) {
            return 3
        }

        if (label.toLowerCase().contains(query.toLowerCase())) {
            return 2
        }

        return 1
    }
}
