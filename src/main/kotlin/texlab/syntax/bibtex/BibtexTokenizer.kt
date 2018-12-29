package texlab.syntax.bibtex

import texlab.syntax.CharStream
import texlab.syntax.TokenSource

class BibtexTokenizer(private val stream: CharStream) : TokenSource<BibtexToken> {
    constructor(text: String) : this(CharStream(text))

    override fun next(): BibtexToken? {
        while (stream.available) {
            val c = stream.peek()
            when (c) {
                '@' -> return type()
                '=' -> return singleCharacter(BibtexTokenKind.ASSIGN)
                ',' -> return singleCharacter(BibtexTokenKind.COMMA)
                '#' -> return singleCharacter(BibtexTokenKind.CONCAT)
                '"' -> return singleCharacter(BibtexTokenKind.QUOTE)
                '{' -> return singleCharacter(BibtexTokenKind.BEGIN_BRACE)
                '}' -> return singleCharacter(BibtexTokenKind.END_BRACE)
                '(' -> return singleCharacter(BibtexTokenKind.BEGIN_PAREN)
                ')' -> return singleCharacter(BibtexTokenKind.END_PAREN)
                '\\' -> command()
                else -> {
                    if (c.isWhitespace()) {
                        stream.next()
                    } else {
                        return word()
                    }
                }
            }
        }
        return null
    }

    private fun type(): BibtexToken {
        fun isTypeChar(c: Char): Boolean = c in 'a'..'z' || c in 'A'..'Z'

        val startPosition = stream.position
        val startIndex = stream.index
        do {
            stream.next()
        } while (stream.available && isTypeChar(stream.peek()))
        val text = stream.text.substring(startIndex, stream.index)
        val kind = when (text.toLowerCase()) {
            "@preamble" ->
                BibtexTokenKind.PREAMBLE_TYPE
            "@string" ->
                BibtexTokenKind.STRING_TYPE
            else ->
                BibtexTokenKind.ENTRY_TYPE
        }
        return BibtexToken(startPosition, text, kind)
    }

    private fun singleCharacter(kind: BibtexTokenKind): BibtexToken {
        val startPosition = stream.position
        stream.next()
        val text = stream.text.substring(stream.index - 1, stream.index)
        return BibtexToken(startPosition, text, kind)
    }

    private fun command(): BibtexToken {
        fun isCommandChar(c: Char): Boolean {
            return c in 'a'..'z' || c in 'A'..'Z' || c == '@'
        }

        val startPosition = stream.position
        val startIndex = stream.index
        stream.next()
        var escape = true
        while (stream.available && isCommandChar(stream.peek())) {
            stream.next()
            escape = false
        }

        if (stream.available && stream.peek() != '\r' && stream.peek() != '\n' &&
                (escape || stream.peek() == '*')) {
            stream.next()
        }

        val text = stream.text.substring(startIndex, stream.index)
        return BibtexToken(startPosition, text, BibtexTokenKind.COMMAND)
    }

    private fun word(): BibtexToken {
        fun isWordChar(c: Char): Boolean {
            return !c.isWhitespace() && c != '@' && c != '=' &&
                    c != ',' && c != '#' && c != '"' && c != '{' &&
                    c != '}' && c != '(' && c != ')'
        }

        val startPosition = stream.position
        val startIndex = stream.index
        do {
            stream.next()
        } while (stream.available && isWordChar(stream.peek()))
        val text = stream.text.substring(startIndex, stream.index)
        return BibtexToken(startPosition, text, BibtexTokenKind.WORD)
    }
}
