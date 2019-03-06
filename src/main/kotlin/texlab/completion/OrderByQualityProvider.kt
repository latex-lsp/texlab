package texlab.completion

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.BibtexDocument
import texlab.LatexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.*
import texlab.syntax.latex.LatexCommandSyntax
import texlab.syntax.latex.LatexDocumentSyntax
import texlab.syntax.latex.LatexGroupSyntax
import texlab.syntax.latex.LatexTextSyntax

class OrderByQualityProvider(private val provider: FeatureProvider<CompletionParams, List<CompletionItem>>)
    : FeatureProvider<CompletionParams, List<CompletionItem>> {

    override suspend fun get(request: FeatureRequest<CompletionParams>): List<CompletionItem> {
        val name = getName(request)
        return if (name == null) {
            emptyList()
        } else {
            provider.get(request)
                    .sortedByDescending { getQuality(it.label, name) }
        }
    }

    private fun getName(request: FeatureRequest<CompletionParams>): String? {
        return when (request.document) {
            is LatexDocument -> {
                val descendants = request.document.tree.root.descendants()
                val node = descendants
                        .filterIsInstance<LatexCommandSyntax>()
                        .lastOrNull { it.name.range.contains(request.params.position) }
                        ?: descendants.lastOrNull { it.range.contains(request.params.position) }

                when (node) {
                    is LatexGroupSyntax -> ""
                    is LatexCommandSyntax -> node.name.text.substring(1)
                    is LatexTextSyntax -> node.words.last().text
                    is LatexDocumentSyntax -> null
                    null -> null
                }
            }
            is BibtexDocument -> {
                val node = request.document.tree.root
                        .descendants()
                        .lastOrNull { it.range.contains(request.params.position) }

                when (node) {
                    is BibtexDocumentSyntax -> ""
                    is BibtexDeclarationSyntax -> {
                        if (node.type.range.contains(request.params.position)) {
                            node.type.text.substring(1)
                        } else {
                            ""
                        }
                    }
                    is BibtexCommentSyntax -> {
                        node.token.text
                    }
                    is BibtexFieldSyntax -> {
                        if (node.name.range.contains(request.params.position)) {
                            node.name.text
                        } else {
                            ""
                        }
                    }
                    is BibtexWordSyntax -> {
                        node.token.text
                    }
                    is BibtexCommandSyntax -> {
                        node.token.text.substring(1)
                    }
                    is BibtexQuotedContentSyntax -> ""
                    is BibtexBracedContentSyntax -> ""
                    is BibtexConcatSyntax -> ""
                    null -> null
                }
            }
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
