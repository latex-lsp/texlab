package texlab.syntax.bibtex

import texlab.syntax.TokenBuffer
import texlab.syntax.bibtex.BibtexTokenKind.*

class BibtexParser(private val tokens: TokenBuffer<BibtexToken>) {
    constructor(text: String) : this(TokenBuffer(BibtexTokenizer(text)))

    fun document(): BibtexDocumentSyntax {
        val children = mutableListOf<BibtexDocumentItemSyntax>()
        while (tokens.available) {
            when (tokens.peek()!!.kind) {
                PREAMBLE_TYPE ->
                    children.add(preamble())
                STRING_TYPE ->
                    children.add(string())
                ENTRY_TYPE ->
                    children.add(entry())
                else -> {
                    val token = tokens.next()
                    children.add(BibtexCommentSyntax(token))
                }
            }
        }
        return BibtexDocumentSyntax(children)
    }

    private fun preamble(): BibtexPreambleSyntax {
        val type = tokens.next()
        val left = expect(BEGIN_BRACE, BEGIN_PAREN) ?: return BibtexPreambleSyntax(type, null, null, null)

        if (!canMatchContent()) {
            return BibtexPreambleSyntax(type, left, null, null)
        }
        val content = content()

        val right = expect(END_BRACE, END_PAREN)
        return BibtexPreambleSyntax(type, left, content, right)
    }

    private fun string(): BibtexStringSyntax {
        val type = tokens.next()

        val left = expect(BEGIN_BRACE, BEGIN_PAREN) ?: return BibtexStringSyntax(type, null, null, null, null, null)
        val name = expect(WORD) ?: return BibtexStringSyntax(type, left, null, null, null, null)
        val assign = expect(ASSIGN) ?: return BibtexStringSyntax(type, left, name, null, null, null)

        if (!canMatchContent()) {
            return BibtexStringSyntax(type, left, name, assign, null, null)
        }
        val value = content()

        val right = expect(END_BRACE, END_PAREN)
        return BibtexStringSyntax(type, left, name, assign, value, right)
    }

    private fun entry(): BibtexEntrySyntax {
        val type = tokens.next()

        val left = expect(BEGIN_BRACE, BEGIN_PAREN)
                ?: return BibtexEntrySyntax(type, null, null, null, emptyList(), null)

        val name = expect(WORD)
                ?: return BibtexEntrySyntax(type, left, null, null, emptyList(), null)

        val comma = expect(COMMA)
                ?: return BibtexEntrySyntax(type, left, name, null, emptyList(), null)

        val fields = mutableListOf<BibtexFieldSyntax>()
        while (tokens.peek()?.kind == WORD) {
            fields.add(field())
        }

        val right = expect(END_BRACE, END_PAREN)
        return BibtexEntrySyntax(type, left, name, comma, fields, right)
    }

    private fun field(): BibtexFieldSyntax {
        val name = tokens.next()
        val assign = expect(ASSIGN) ?: return BibtexFieldSyntax(name, null, null, null)

        if (!canMatchContent()) {
            return BibtexFieldSyntax(name, assign, null, null)
        }
        val content = content()

        val comma = expect(COMMA)
        return BibtexFieldSyntax(name, assign, content, comma)
    }

    private fun content(inQuotes: Boolean = false): BibtexContentSyntax {
        val token = tokens.next()
        val left = when (token.kind) {
            PREAMBLE_TYPE -> BibtexWordSyntax(token)
            STRING_TYPE -> BibtexWordSyntax(token)
            ENTRY_TYPE -> BibtexWordSyntax(token)
            WORD -> BibtexWordSyntax(token)
            COMMAND -> BibtexCommandSyntax(token)
            ASSIGN -> BibtexWordSyntax(token)
            COMMA -> BibtexWordSyntax(token)
            CONCAT -> TODO()
            QUOTE -> {
                val children = mutableListOf<BibtexContentSyntax>()
                while (canMatchContent()) {
                    val kind = tokens.peek()!!.kind
                    if (kind == QUOTE && inQuotes) {
                        break
                    }
                    children.add(content(true))
                }
                val right = expect(QUOTE)
                BibtexQuotedContentSyntax(token, children, right)
            }
            BEGIN_BRACE -> {
                val children = mutableListOf<BibtexContentSyntax>()
                while (canMatchContent()) {
                    children.add(content(false))
                }
                val right = expect(END_BRACE)
                BibtexBracedContentSyntax(token, children, right)
            }
            END_BRACE -> TODO()
            BEGIN_PAREN -> BibtexWordSyntax(token)
            END_PAREN -> BibtexWordSyntax(token)
        }

        val operator = expect(CONCAT)
        return if (operator == null) {
            left
        } else {
            val right = if (canMatchContent()) {
                content()
            } else {
                null
            }
            BibtexConcatSyntax(left, operator, right)
        }
    }

    private fun canMatchContent(): Boolean {
        val token = tokens.peek()
        return if (token == null) {
            false
        } else {
            when (token.kind) {
                PREAMBLE_TYPE -> true
                STRING_TYPE -> true
                ENTRY_TYPE -> true
                WORD -> true
                COMMAND -> true
                ASSIGN -> true
                COMMA -> true
                CONCAT -> false
                QUOTE -> true
                BEGIN_BRACE -> true
                END_BRACE -> false
                BEGIN_PAREN -> true
                END_PAREN -> true
            }
        }
    }

    private fun expect(vararg kinds: BibtexTokenKind): BibtexToken? {
        val token = tokens.peek()
        return if (token != null && kinds.any { it == token.kind }) {
            tokens.next()
        } else {
            null
        }
    }

    companion object {
        fun parse(text: String): BibtexDocumentSyntax {
            return BibtexParser(text).document()
        }
    }
}
