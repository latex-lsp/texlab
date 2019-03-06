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

class MatchQualityComparator(private val document: Document,
                             private val position: Position) : Comparator<CompletionItem> {
    override fun compare(left: CompletionItem, right: CompletionItem): Int {
        val leftQuality = getQuality(left.label)
        val rightQuality = getQuality(right.label)
        return rightQuality.compareTo(leftQuality)
    }

    private fun getQuality(label: String): Int {
        val query = getName(document, position) ?: return -1
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

    private fun getName(document: Document, position: Position): String? {
        return when (document) {
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
    }
}
