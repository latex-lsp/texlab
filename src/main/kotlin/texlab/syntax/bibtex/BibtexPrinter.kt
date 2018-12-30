package texlab.syntax.bibtex

class BibtexPrinter private constructor() {

    private val builder: StringBuilder = StringBuilder()
    private var line: Int = 0
    private var character: Int = 0

    private fun visit(node: BibtexSyntaxNode) {
        when (node) {
            is BibtexDocumentSyntax -> {
                node.children.forEach { visit(it) }
            }
            is BibtexCommentSyntax -> {
                visit(node.token)
            }
            is BibtexPreambleSyntax -> {
                visit(node.type)
                node.left?.also { visit(it) }
                node.content?.also { visit(it) }
                node.right?.also { visit(it) }
            }
            is BibtexStringSyntax -> {
                visit(node.type)
                node.left?.also { visit(it) }
                node.name?.also { visit(it) }
                node.assign?.also { visit(it) }
                node.value?.also { visit(it) }
                node.right?.also { visit(it) }
            }
            is BibtexEntrySyntax -> {
                visit(node.type)
                node.left?.also { visit(it) }
                node.name?.also { visit(it) }
                node.comma?.also { visit(it) }
                node.fields.forEach { visit(it) }
                node.right?.also { visit(it) }
            }
            is BibtexFieldSyntax -> {
                visit(node.name)
                node.assign?.also { visit(it) }
                node.content?.also { visit(it) }
                node.comma?.also { visit(it) }
            }
            is BibtexWordSyntax -> {
                visit(node.token)
            }
            is BibtexCommandSyntax -> {
                visit(node.token)
            }
            is BibtexQuotedContentSyntax -> {
                visit(node.left)
                node.children.forEach { visit(it) }
                node.right?.also { visit(it) }
            }
            is BibtexBracedContentSyntax -> {
                visit(node.left)
                node.children.forEach { visit(it) }
                node.right?.also { visit(it) }
            }
            is BibtexConcatSyntax -> {
                visit(node.left)
                visit(node.operator)
                node.right?.also { visit(it) }
            }
        }
    }

    private fun visit(token: BibtexToken) {
        while (line < token.line) {
            builder.appendln()
            line++
            character = 0
        }

        while (character < token.character) {
            builder.append(" ")
            character++
        }

        builder.append(token.text)
        character += token.text.length
    }

    override fun toString(): String = builder.toString()

    companion object {
        fun print(tree: BibtexSyntaxNode): String {
            return BibtexPrinter().run {
                visit(tree)
                toString()
            }
        }
    }
}
