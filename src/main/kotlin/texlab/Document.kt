package texlab

import org.eclipse.lsp4j.TextDocumentContentChangeEvent
import texlab.syntax.CharStream
import texlab.syntax.latex.LatexSyntaxTree
import java.net.URI

sealed class Document(val uri: URI) {

    private var version: Int = -1

    var text: String = ""
        private set

    val isFile: Boolean = uri.scheme == "file"

    fun update(changes: List<TextDocumentContentChangeEvent>, version: Int) {
        if (this.version > version) {
            return
        }

        changes.forEach { change ->
            text = if (change.range == null) {
                change.text
            } else {
                val stream = CharStream(text)
                stream.seek(change.range.start)
                val left = text.substring(0, stream.index)
                stream.seek(change.range.end)
                val right = text.substring(stream.index)
                left + change.text + right
            }
        }
        this.version = version
        analyze()
    }

    protected abstract fun analyze()
}

class LatexDocument(uri: URI) : Document(uri) {

    var tree: LatexSyntaxTree = LatexSyntaxTree(text)

    override fun analyze() {
        tree = LatexSyntaxTree(text)
    }
}

class BibtexDocument(uri: URI) : Document(uri) {

    override fun analyze() {
        // TODO
    }
}
