package texlab.syntax.bibtex

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import texlab.syntax.SyntaxNode

sealed class BibtexSyntaxNode : SyntaxNode()

data class BibtexDocumentSyntax(val children: List<BibtexDeclarationSyntax>) : BibtexSyntaxNode() {
    override val range = if (children.isEmpty()) {
        Range(Position(0, 0), Position(0, 0))
    } else {
        Range(children.first().start, children.last().end)
    }
}

sealed class BibtexDeclarationSyntax : BibtexSyntaxNode()

data class BibtexCommentSyntax(val token: BibtexToken) : BibtexDeclarationSyntax() {
    override val range = token.range
}

data class BibtexPreambleSyntax(val type: BibtexToken,
                                val left: BibtexToken?,
                                val content: BibtexContentSyntax?,
                                val right: BibtexToken?) : BibtexDeclarationSyntax() {
    override val range = Range(type.start, right?.end ?: content?.end ?: left?.end ?: type.end)
}

data class BibtexStringSyntax(val type: BibtexToken,
                              val left: BibtexToken?,
                              val name: BibtexToken?,
                              val assign: BibtexToken?,
                              val value: BibtexContentSyntax?,
                              val right: BibtexToken?) : BibtexDeclarationSyntax() {
    override val range = Range(type.start,
            right?.end ?: value?.end ?: assign?.end ?: name?.end ?: left?.end ?: type.end)
}

data class BibtexEntrySyntax(val type: BibtexToken,
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
