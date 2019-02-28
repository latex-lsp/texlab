package texlab

import texlab.syntax.BibtexSyntaxTree
import texlab.syntax.LatexSyntaxTree
import texlab.syntax.SyntaxTree
import java.net.URI

sealed class Document {
    abstract val uri: URI
    abstract val text: String
    abstract val tree: SyntaxTree

    val isFile: Boolean
        get() = uri.scheme == "file"

    abstract fun copy(text: String = this.text, tree: SyntaxTree = this.tree): Document

    companion object {
        fun create(uri: URI, text: String, language: Language): Document {
            return when (language) {
                Language.LATEX -> {
                    LatexDocument(uri, text, LatexSyntaxTree(text))
                }
                Language.BIBTEX ->
                    BibtexDocument(uri, text, BibtexSyntaxTree(text))
            }
        }
    }
}

data class LatexDocument(override val uri: URI,
                         override val text: String,
                         override val tree: LatexSyntaxTree) : Document() {
    override fun copy(text: String, tree: SyntaxTree): Document =
            copy(uri = uri, text = text, tree = tree as LatexSyntaxTree)
}

data class BibtexDocument(override val uri: URI,
                          override val text: String,
                          override val tree: BibtexSyntaxTree) : Document() {
    override fun copy(text: String, tree: SyntaxTree): Document =
            copy(uri = uri, text = text, tree = tree as BibtexSyntaxTree)
}
