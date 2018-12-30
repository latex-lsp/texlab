package texlab.syntax.bibtex

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import texlab.syntax.SyntaxNode

sealed class BibtexSyntaxNode : SyntaxNode() {
    fun descendants(): List<BibtexSyntaxNode> {
        val results = mutableListOf<BibtexSyntaxNode>()

        fun visit(node: BibtexSyntaxNode) {
            results.add(node)
            when (node) {
                is BibtexDocumentSyntax -> {
                    node.children.forEach { visit(it) }
                }
                is BibtexCommentSyntax -> {
                }
                is BibtexPreambleSyntax -> {
                    node.content?.also { visit(it) }
                }
                is BibtexStringSyntax -> {
                    node.value?.also { visit(it) }
                }
                is BibtexEntrySyntax -> {
                    node.fields.forEach { visit(it) }
                }
                is BibtexFieldSyntax -> {
                    node.content?.also { visit(it) }
                }
                is BibtexWordSyntax -> {
                }
                is BibtexCommandSyntax -> {
                }
                is BibtexQuotedContentSyntax -> {
                    node.children.forEach { visit(it) }
                }
                is BibtexBracedContentSyntax -> {
                    node.children.forEach { visit(it) }
                }
                is BibtexConcatSyntax -> {
                    visit(node.left)
                    node.right?.also { visit(it) }
                }
            }
        }

        visit(this)
        return results
    }
}

data class BibtexDocumentSyntax(val children: List<BibtexDocumentItemSyntax>) : BibtexSyntaxNode() {
    override val range = if (children.isEmpty()) {
        Range(Position(0, 0), Position(0, 0))
    } else {
        Range(children.first().start, children.last().end)
    }
}

sealed class BibtexDocumentItemSyntax : BibtexSyntaxNode()

sealed class BibtexDeclarationSyntax : BibtexDocumentItemSyntax() {
    abstract val type: BibtexToken
}

data class BibtexCommentSyntax(val token: BibtexToken) : BibtexDocumentItemSyntax() {
    override val range = token.range
}

data class BibtexPreambleSyntax(override val type: BibtexToken,
                                val left: BibtexToken?,
                                val content: BibtexContentSyntax?,
                                val right: BibtexToken?) : BibtexDeclarationSyntax() {
    override val range = Range(type.start, right?.end ?: content?.end ?: left?.end ?: type.end)
}

data class BibtexStringSyntax(override val type: BibtexToken,
                              val left: BibtexToken?,
                              val name: BibtexToken?,
                              val assign: BibtexToken?,
                              val value: BibtexContentSyntax?,
                              val right: BibtexToken?) : BibtexDeclarationSyntax() {
    override val range = Range(type.start,
            right?.end ?: value?.end ?: assign?.end ?: name?.end ?: left?.end ?: type.end)
}

data class BibtexEntrySyntax(override val type: BibtexToken,
                             val left: BibtexToken?,
                             val name: BibtexToken?,
                             val comma: BibtexToken?,
                             val fields: List<BibtexFieldSyntax>,
                             val right: BibtexToken?) : BibtexDeclarationSyntax() {
    override val range = Range(type.start,
            right?.end ?: fields.lastOrNull()?.end ?: comma?.end ?: name?.end ?: left?.end ?: type.end)
}

data class BibtexFieldSyntax(val name: BibtexToken,
                             val assign: BibtexToken?,
                             val content: BibtexContentSyntax?,
                             val comma: BibtexToken?) : BibtexSyntaxNode() {
    override val range = Range(name.start, comma?.end ?: content?.end ?: assign?.end ?: name.end)
}

sealed class BibtexContentSyntax : BibtexSyntaxNode()

data class BibtexWordSyntax(val token: BibtexToken) : BibtexContentSyntax() {
    override val range = token.range
}

data class BibtexCommandSyntax(val token: BibtexToken) : BibtexContentSyntax() {
    override val range = token.range
}

data class BibtexQuotedContentSyntax(val left: BibtexToken,
                                     val children: List<BibtexContentSyntax>,
                                     val right: BibtexToken?) : BibtexContentSyntax() {
    override val range = Range(left.start, right?.end ?: children.lastOrNull()?.end ?: left.end)
}

data class BibtexBracedContentSyntax(val left: BibtexToken,
                                     val children: List<BibtexContentSyntax>,
                                     val right: BibtexToken?) : BibtexContentSyntax() {
    override val range = Range(left.start, right?.end ?: children.lastOrNull()?.end ?: left.end)
}

data class BibtexConcatSyntax(val left: BibtexContentSyntax,
                              val operator: BibtexToken,
                              val right: BibtexContentSyntax?) : BibtexContentSyntax() {
    override val range = Range(left.start, right?.end ?: operator.end)
}
