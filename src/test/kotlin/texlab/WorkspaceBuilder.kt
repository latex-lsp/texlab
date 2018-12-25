package texlab

import org.eclipse.lsp4j.Position
import texlab.completion.CompletionRequest
import java.io.File

class WorkspaceBuilder {
    val workspace = Workspace()

    fun document(name: String, text: String): WorkspaceBuilder {
        val file = File(name)
        val language = getLanguageByExtension(file.extension)!!
        val document = Document.create(file.toURI(), language)
        document.text = text
        document.analyze()
        workspace.documents.add(document)
        return this
    }

    fun completion(name: String, line: Int, character: Int): CompletionRequest {
        val uri = File(name).toURI()
        val position = Position(line, character)
        return CompletionRequest(uri, workspace.relatedDocuments(uri), position)
    }
}
