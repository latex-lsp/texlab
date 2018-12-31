package texlab

import texlab.syntax.bibtex.*

object BibtexFormatter {
    fun format(builder: StringBuilder, entry: BibtexEntrySyntax, settings: BibtexFormatterSettings) {
        builder.append(entry.type.text)
        builder.append("{")
        builder.append(entry.name?.text ?: return)
        builder.append(",")
        builder.appendln()
        for (field in entry.fields) {
            format(builder, field, settings)
        }
        builder.append("}")
    }

    private fun format(builder: StringBuilder, field: BibtexFieldSyntax, settings: BibtexFormatterSettings) {
        builder.append(settings.indent)
        builder.append(field.name.text)
        builder.append(" = ")
        val startLength = settings.tabSize + field.name.text.length + 3

        val tokens = getTokens(field.content ?: return)
        builder.append(tokens[0].text)

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
                builder.appendln()
                builder.append(settings.indent)
                for (j in 0 until startLength - settings.tabSize + 1) {
                    builder.append(" ")
                }
                length = startLength
            } else if (insertSpace) {
                builder.append(" ")
                length++
            }
            builder.append(current.text)
            length += current.length
        }
        builder.appendln(",")
    }

    private fun shouldInsertSpace(previous: BibtexToken, current: BibtexToken): Boolean {
        if (previous.line != current.line) {
            return true
        }

        return previous.end.character < current.start.character
    }

    private fun getTokens(content: BibtexContentSyntax): List<BibtexToken> {
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
