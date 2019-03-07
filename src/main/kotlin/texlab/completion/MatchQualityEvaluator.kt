package texlab.completion

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.Position
import texlab.BibtexDocument
import texlab.Document
import texlab.LatexDocument
import texlab.contains
import texlab.syntax.bibtex.*
import texlab.syntax.latex.LatexCommandSyntax
import texlab.syntax.latex.LatexDocumentSyntax
import texlab.syntax.latex.LatexGroupSyntax
import texlab.syntax.latex.LatexTextSyntax

class MatchQualityEvaluator(document: Document, private val position: Position) {
    private val query: String? = when (document) {
        is LatexDocument -> {
            val descendants = document.tree.root.descendants()
            val node = descendants
                    .filterIsInstance<LatexCommandSyntax>()
                    .lastOrNull { it.name.range.contains(position) }
                    ?: descendants.lastOrNull { it.range.contains(position) }

            when (node) {
                is LatexGroupSyntax -> ""
                is LatexCommandSyntax -> node.name.text.substring(1)
                is LatexTextSyntax -> node.words.last().text
                is LatexDocumentSyntax -> null
                null -> null
            }
        }
        is BibtexDocument -> {
            val node = document.tree.root
                    .descendants()
                    .lastOrNull { it.range.contains(position) }

            when (node) {
                is BibtexDocumentSyntax -> ""
                is BibtexDeclarationSyntax -> {
                    if (node.type.range.contains(position)) {
                        node.type.text.substring(1)
                    } else {
                        ""
                    }
                }
                is BibtexCommentSyntax -> {
                    node.token.text
                }
                is BibtexFieldSyntax -> {
                    if (node.name.range.contains(position)) {
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

    fun evaluate(item: CompletionItem): Int {
        if (query == null) {
            return -1
        }

        val label = item.label
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
