package texlab

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.TextDocumentContentChangeEvent
import org.eclipse.lsp4j.WorkspaceEdit
import java.net.URI

abstract class Document(val uri: URI) {

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

    abstract fun documentSymbol(): List<DocumentSymbol>

    abstract fun rename(documents: List<Document>, position: Position, newName: String): WorkspaceEdit?
}
