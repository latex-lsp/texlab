package texlab

import texlab.syntax.latex.LatexSyntaxTree
import java.net.URI

sealed class Document(val uri: URI) {
    val isFile: Boolean = uri.scheme == "file"

    var text: String = ""

    var version: Int = -1

    override fun equals(other: Any?): Boolean {
        return other is Document && uri == other.uri
    }

    override fun hashCode(): Int = uri.hashCode()

    abstract fun analyze()

    companion object {
        fun create(uri: URI, language: Language): Document {
            return when (language) {
                Language.LATEX ->
                    LatexDocument(uri)
                Language.BIBTEX ->
                    BibtexDocument(uri)
            }
        }
    }
}

class LatexDocument(uri: URI) : Document(uri) {
    var tree: LatexSyntaxTree = LatexSyntaxTree(text)
        private set

    override fun analyze() {
        tree = LatexSyntaxTree(text)
    }
}

class BibtexDocument(uri: URI) : Document(uri) {
    override fun analyze() {
        // TODO
    }
}
