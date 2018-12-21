package texlab

import org.eclipse.lsp4j.TextDocumentContentChangeEvent
import java.net.URI

abstract class Document(val uri: URI) {

    var version: Int = -1
        private set

    var text: String = ""
        private set

    protected abstract fun analyze()

    fun update(changes: List<TextDocumentContentChangeEvent>, version: Int) {
        if (version > this.version) {
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
}
