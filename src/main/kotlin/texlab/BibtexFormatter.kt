package texlab

import texlab.syntax.bibtex.*

class BibtexFormatter(private val settings: BibtexFormatterSettings) {
    fun format(entry: BibtexEntrySyntax): String = buildString {
        append(entry.type.text.toLowerCase())
        append("{")
        append(entry.name?.text ?: return@buildString)
        appendln(",")
        entry.fields.forEach { append(format(it)) }
        append("}")
    }

    private fun format(field: BibtexFieldSyntax): String = buildString {
        append(settings.indent)
        append(field.name.text.toLowerCase())
        append(" = ")
        val startLength = settings.tabSize + field.name.text.length + 3

        val tokens = descendants(field.content ?: return@buildString)
        append(tokens[0].text)

        var length = startLength + tokens[0].length
        for (i in 1 until tokens.size) {
            val previous = tokens[i - 1]
            val current = tokens[i]

            val insertSpace = shouldInsertSpace(previous, current)
            val spaceLength = if (insertSpace) {
                1
            } else {
                0
            }

            if (length + current.length + spaceLength > settings.lineLength) {
                appendln()
                append(settings.indent)
                for (j in 0 until startLength - settings.tabSize + 1) {
                    append(" ")
                }
                length = startLength
            } else if (insertSpace) {
                append(" ")
                length++
            }
            append(current.text)
            length += current.length
        }
        appendln(",")
    }

    private fun shouldInsertSpace(previous: BibtexToken, current: BibtexToken): Boolean {
        if (previous.line != current.line) {
            return true
        }

        return previous.end.character < current.start.character
    }

    private fun descendants(content: BibtexContentSyntax): List<BibtexToken> {
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
