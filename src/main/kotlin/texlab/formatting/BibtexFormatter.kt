package texlab.formatting

import texlab.syntax.bibtex.*
import java.util.*

class BibtexFormatter(insertSpaces: Boolean,
                      private val tabSize: Int,
                      private var lineLength: Int) {
    init {
        lineLength = if (lineLength <= 0) {
            Int.MAX_VALUE
        } else {
            lineLength
        }
    }

    private val indent: String = if (insertSpaces) {
        Collections.nCopies(tabSize, " ").joinToString("")
    } else {
        "\t"
    }

    fun format(declaration: BibtexDeclarationSyntax): String {
        return when (declaration) {
            is BibtexPreambleSyntax ->
                format(declaration)
            is BibtexStringSyntax ->
                format(declaration)
            is BibtexEntrySyntax ->
                format(declaration)
        }
    }

    fun format(preamble: BibtexPreambleSyntax): String = buildString {
        append(format(preamble.type))
        append("{")
        append(format(preamble.content ?: return@buildString, length))
        append("}")
    }

    fun format(string: BibtexStringSyntax): String = buildString {
        append(format(string.type))
        append("{")
        append(string.name?.text)
        append(" = ")
        append(format(string.value ?: return@buildString, length))
        append("}")
    }

    fun format(entry: BibtexEntrySyntax): String = buildString {
        append(format(entry.type))
        append("{")
        append(entry.name?.text ?: return@buildString)
        appendln(",")
        entry.fields.forEach { append(format(it)) }
        append("}")
    }

    fun format(field: BibtexFieldSyntax): String = buildString {
        append(indent)
        append(format(field.name))
        append(" = ")
        val align = tabSize + field.name.text.length + 3
        append(format(field.content ?: return@buildString, align))
        appendln(",")
    }

    fun format(content: BibtexContentSyntax, align: Int): String = buildString {
        val tokens = getAllTokens(content)
        append(tokens[0].text)

        var length = align + tokens[0].length
        for (i in 1 until tokens.size) {
            val previous = tokens[i - 1]
            val current = tokens[i]

            val insertSpace = shouldInsertSpace(previous, current)
            val spaceLength = if (insertSpace) {
                1
            } else {
                0
            }

            if (length + current.length + spaceLength > lineLength) {
                appendln()
                append(indent)
                for (j in 0 until align - tabSize + 1) {
                    append(" ")
                }
                length = align
            } else if (insertSpace) {
                append(" ")
                length++
            }
            append(current.text)
            length += current.length
        }
    }

    private fun format(token: BibtexToken): String {
        return token.text.toLowerCase()
    }

    private fun shouldInsertSpace(previous: BibtexToken, current: BibtexToken): Boolean {
        return previous.line != current.line ||
                previous.end.character < current.start.character
    }

    private fun getAllTokens(content: BibtexContentSyntax): List<BibtexToken> {
        val tokens = mutableListOf<BibtexToken>()
        fun visit(node: BibtexContentSyntax) {
            when (node) {
                is BibtexWordSyntax -> {
                    tokens.add(node.token)
                }
                is BibtexCommandSyntax -> {
                    tokens.add(node.token)
                }
                is BibtexQuotedContentSyntax -> {
                    tokens.add(node.left)
                    node.children.forEach { visit(it) }
                    node.right?.also { tokens.add(it) }
                }
                is BibtexBracedContentSyntax -> {
                    tokens.add(node.left)
                    node.children.forEach { visit(it) }
                    node.right?.also { tokens.add(it) }
                }
                is BibtexConcatSyntax -> {
                    visit(node.left)
                    tokens.add(node.operator)
                    node.right?.also { visit(it) }
                }
            }
        }
        visit(content)
        return tokens
    }
}
