package texlab.latex

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range

sealed class LatexSyntaxNode {
    abstract val range: Range

    val start: Position
        get() = range.start

    val end: Position
        get() = range.end

    fun descendants(): List<LatexSyntaxNode> {
        val results = mutableListOf<LatexSyntaxNode>()

        fun visit(node: LatexSyntaxNode) {
            results.add(node)
            when (node) {
                is LatexDocumentSyntax -> {
                    node.children.forEach { visit(it) }
                }
                is LatexGroupSyntax -> {
                    node.children.forEach { visit(it) }
                }
                is LatexCommandSyntax -> {
                    node.options?.also { visit(it) }
                    node.args.forEach { visit(it) }
                }
            }
        }

        visit(this)
        return results
    }
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
                              val args: List<LatexGroupSyntax>) : LatexSyntaxNode() {

    fun extractText(index: Int): LatexTextSyntax? {
        return if (args.size > index && args[index].children.size == 1) {
            val child = args[index].children[0]
            if (child is LatexTextSyntax) {
                child
            } else {
                null
            }
        } else {
            null
        }
    }

    fun extractWord(index: Int): String? {
        val text = extractText(index)
        return if (text == null || text.words.size != 1) {
            null
        } else {
            text.words[0].text
        }
    }
}


data class LatexTextSyntax(override val range: Range,
                           val words: List<LatexToken>) : LatexSyntaxNode()
