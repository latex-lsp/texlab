package texlab.syntax.latex

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import texlab.syntax.SyntaxNode

sealed class LatexSyntaxNode : SyntaxNode() {
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
                is LatexTextSyntax -> {
                }
            }
        }

        visit(this)
        return results
    }
}

data class LatexDocumentSyntax(val children: List<LatexSyntaxNode>) : LatexSyntaxNode() {
    override val range = if (children.isEmpty()) {
        Range(Position(0, 0), Position(0, 0))
    } else {
        Range(children.first().start, children.last().end)
    }
}

data class LatexGroupSyntax(val left: LatexToken,
                            val children: List<LatexSyntaxNode>,
                            val right: LatexToken?) : LatexSyntaxNode() {
    override val range = Range(left.start, right?.end ?: children.lastOrNull()?.end ?: left.end)
}

data class LatexCommandSyntax(val name: LatexToken,
                              val options: LatexGroupSyntax?,
                              val args: List<LatexGroupSyntax>) : LatexSyntaxNode() {
    override val range = Range(name.start, args.lastOrNull()?.end ?: options?.end ?: name.end)

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


data class LatexTextSyntax(val words: List<LatexToken>) : LatexSyntaxNode() {
    override val range = Range(words.first().start, words.last().end)
}
