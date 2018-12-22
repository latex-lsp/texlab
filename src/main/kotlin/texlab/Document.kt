package texlab

import org.eclipse.lsp4j.*
import texlab.syntax.CharStream
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

    abstract fun documentSymbol(workspace: Workspace): List<DocumentSymbol>

    abstract fun documentLink(workspace: Workspace): List<DocumentLink>

    abstract fun rename(workspace: Workspace, position: Position, newName: String): WorkspaceEdit?
}
