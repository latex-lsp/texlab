package texlab.syntax.latex

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import texlab.syntax.TokenBuffer

class LatexParser(private val tokens: TokenBuffer<LatexToken>) {

    constructor(text: String) : this(TokenBuffer(LatexTokenizer(text)))

    fun document(): LatexDocumentSyntax {
        val children = content(LatexScope.DOCUMENT)
        val range = if (children.isEmpty()) {
            Range(Position(0, 0), Position(0, 0))
        } else {
            Range(children.first().start, children.last().end)
        }
        return LatexDocumentSyntax(range, children)
    }

    private fun content(scope: LatexScope): List<LatexSyntaxNode> {
        val children = mutableListOf<LatexSyntaxNode>()
        while (tokens.available) {
            when (tokens.peek()!!.kind) {
                LatexTokenKind.WORD -> {
                    children.add(text(scope))
                }
                LatexTokenKind.COMMAND -> {
                    children.add(command())
                }
                LatexTokenKind.BEGIN_GROUP -> {
                    children.add(group(LatexScope.GROUP))
                }
                LatexTokenKind.END_GROUP -> {
                    if (scope == LatexScope.DOCUMENT) {
                        tokens.next()
                    } else {
                        return children
                    }
                }
                LatexTokenKind.BEGIN_OPTIONS -> {
                    children.add(text(scope))
                }
                LatexTokenKind.END_OPTIONS -> {
                    if (scope == LatexScope.OPTIONS) {
                        return children
                    } else {
                        children.add(text(scope))
                    }
                }
            }
        }
        return children
    }

    private fun group(scope: LatexScope): LatexGroupSyntax {
        val left = tokens.next()
        val children = content(scope)
        val endKind = if (scope == LatexScope.GROUP) {
            LatexTokenKind.END_GROUP
        } else {
            LatexTokenKind.END_OPTIONS
        }

        val right = if (tokens.peek()?.kind == endKind) {
            tokens.next()
        } else {
            null
        }

        val end = right?.end ?: children.lastOrNull()?.end ?: left.end
        val range = Range(left.start, end)
        return LatexGroupSyntax(range, left, right, children)
    }

    private fun command(): LatexCommandSyntax {
        val name = tokens.next()
        val options = if (tokens.peek()?.kind == LatexTokenKind.BEGIN_OPTIONS) {
            group(LatexScope.OPTIONS)
        } else {
            null
        }

        val args = mutableListOf<LatexGroupSyntax>()
        while (tokens.peek()?.kind == LatexTokenKind.BEGIN_GROUP) {
            args.add(group(LatexScope.GROUP))
        }

        val end = args.lastOrNull()?.end ?: options?.end ?: name.end
        val range = Range(name.start, end)
        return LatexCommandSyntax(range, name, options, args)
    }

    private fun text(scope: LatexScope): LatexTextSyntax {
        val words = mutableListOf<LatexToken>()
        while (tokens.available) {
            val kind = tokens.peek()!!.kind
            val opts = kind == LatexTokenKind.END_OPTIONS && scope != LatexScope.OPTIONS
            if (kind == LatexTokenKind.WORD || kind == LatexTokenKind.BEGIN_OPTIONS || opts) {
                words.add(tokens.next())
            } else {
                break
            }
        }
        val range = Range(words.first().start, words.last().end)
        return LatexTextSyntax(range, words)
    }

    companion object {
        fun parse(text: String): LatexDocumentSyntax {
            return LatexParser(text).document()
        }
    }
}
