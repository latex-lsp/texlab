package texlab.latex

import texlab.CharStream
import texlab.TokenSource

class LatexTokenizer(private val stream: CharStream) : TokenSource<LatexToken> {
    constructor(text: String) : this(CharStream(text))

    override fun next(): LatexToken? {
        while (stream.available) {
            val c = stream.peek()
            when (c) {
                '%' -> {
                    stream.next()
                    stream.skipRestOfLine()
                }
                '{' -> return delimiter(LatexTokenKind.BEGIN_GROUP)
                '}' -> return delimiter(LatexTokenKind.END_GROUP)
                '[' -> return delimiter(LatexTokenKind.BEGIN_OPTIONS)
                ']' -> return delimiter(LatexTokenKind.END_OPTIONS)
                '\\' -> return command()
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

    private fun delimiter(kind: LatexTokenKind): LatexToken {
        val startPosition = stream.position
        stream.next()
        val text = stream.text.substring(stream.index - 1, stream.index)
        return LatexToken(startPosition, text, kind)
    }

    private fun command(): LatexToken {
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
        return LatexToken(startPosition, text, LatexTokenKind.COMMAND)
    }

    private fun word(): LatexToken {
        fun isWordChar(c: Char): Boolean {
            return !c.isWhitespace() && c != '%' && c != '{' &&
                    c != '}' && c != '[' && c != ']' && c != '\\'
        }

        val startPosition = stream.position
        val startIndex = stream.index
        do {
            stream.next()
        } while (stream.available && isWordChar(stream.peek()))

        val text = stream.text.substring(startIndex, stream.index)
        return LatexToken(startPosition, text, LatexTokenKind.WORD)
    }
}
