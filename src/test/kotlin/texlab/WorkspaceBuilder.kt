package texlab

import org.eclipse.lsp4j.Position
import texlab.completion.CompletionRequest
import texlab.definition.DefinitionRequest
import texlab.diagnostics.DiagnosticsRequest
import texlab.folding.FoldingRequest
import texlab.highlight.HighlightRequest
import texlab.link.LinkRequest
import texlab.references.ReferenceRequest
import texlab.rename.RenameRequest
import java.io.File

class WorkspaceBuilder {
    val workspace = Workspace()

    fun document(path: String, text: String): WorkspaceBuilder {
        val file = File(path)
        val language = getLanguageByExtension(file.extension)!!
        val document = Document.create(file.toURI(), language)
        document.text = text
        document.analyze()
        workspace.documents.add(document)
        return this
    }

    fun completion(path: String, line: Int, character: Int): CompletionRequest {
        val uri = File(path).toURI()
        val position = Position(line, character)
        return CompletionRequest(uri, workspace.relatedDocuments(uri), position)
    }

    fun definition(path: String, line: Int, character: Int): DefinitionRequest {
        val uri = File(path).toURI()
        val position = Position(line, character)
        return DefinitionRequest(uri, workspace.relatedDocuments(uri), position)
    }

    fun diagnostics(path: String): DiagnosticsRequest {
        val uri = File(path).toURI()
        return DiagnosticsRequest(uri, workspace.relatedDocuments(uri))
    }

    fun folding(path: String): FoldingRequest {
        val uri = File(path).toURI()
        return FoldingRequest(workspace.documents.first { it.uri == uri })
    }

    fun highlight(path: String, line: Int, character: Int): HighlightRequest {
        val uri = File(path).toURI()
        val position = Position(line, character)
        return HighlightRequest(workspace.documents.first { it.uri == uri }, position)
    }

    fun link(path: String): LinkRequest {
        val uri = File(path).toURI()
        return LinkRequest(workspace, uri)
    }

    fun reference(path: String, line: Int, character: Int): ReferenceRequest {
        val uri = File(path).toURI()
        val position = Position(line, character)
        return ReferenceRequest(uri, workspace.relatedDocuments(uri), position)
    }

    fun rename(path: String, line: Int, character: Int, newName: String): RenameRequest {
        val uri = File(path).toURI()
        val relatedDocuments = workspace.relatedDocuments(uri)
        val position = Position(line, character)
        return RenameRequest(uri, relatedDocuments, position, newName)
    }
}
