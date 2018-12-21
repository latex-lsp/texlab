package texlab.latex

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range

sealed class LatexSyntaxNode {
    abstract val range: Range

    val start: Position
        get() = range.start

    val end: Position
        get() = range.end
}

data class LatexDocumentSyntax(override val range: Range,
                               val children: List<LatexSyntaxNode>) : LatexSyntaxNode()

data class LatexGroupSyntax(override val range: Range,
                            val left: LatexToken,
                            val right: LatexToken?,
                            val children: List<LatexSyntaxNode>) : LatexSyntaxNode()

data class LatexCommandSyntax(override val range: Range,
                              val name: LatexToken,
                              val options: LatexGroupSyntax?,
                              val args: List<LatexGroupSyntax>) : LatexSyntaxNode()

data class LatexTextSyntax(override val range: Range,
                           val words: List<LatexToken>) : LatexSyntaxNode()
