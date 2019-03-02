package texlab

import org.eclipse.lsp4j.*
import texlab.provider.FeatureRequest
import java.io.File

class WorkspaceBuilder {
    var workspace = Workspace()

    fun document(path: String, text: String): WorkspaceBuilder {
        val file = File(path)
        val language = getLanguageByExtension(file.extension)!!
        val document = Document.create(file.toURI(), text, language)
        workspace = Workspace(workspace.documents.plus(document))
        return this
    }

    fun <T> request(path: String, paramsFactory: (TextDocumentIdentifier) -> T): FeatureRequest<T> {
        val uri = File(path).toURI()
        val identifier = TextDocumentIdentifier(uri.toString())
        val params = paramsFactory(identifier)
        return FeatureRequest(uri, workspace, params)
    }


    fun <T> positionRequest(path: String,
                            line: Int,
                            character: Int,
                            paramsFactory: (TextDocumentIdentifier, Position) -> T): FeatureRequest<T> {
        val uri = File(path).toURI()
        val position = Position(line, character)
        val identifier = TextDocumentIdentifier(uri.toString())
        val params = paramsFactory(identifier, position)
        return FeatureRequest(uri, workspace, params)
    }

    fun completion(path: String, line: Int, character: Int): FeatureRequest<CompletionParams> {
        return positionRequest(path, line, character) { identifier, position ->
            CompletionParams(identifier, position)
        }
    }

    fun definition(path: String, line: Int, character: Int): FeatureRequest<TextDocumentPositionParams> {
        return positionRequest(path, line, character) { identifier, position ->
            TextDocumentPositionParams(identifier, position)
        }
    }

    fun diagnostics(path: String): FeatureRequest<Unit> {
        return request(path) { _ -> Unit }
    }

    fun folding(path: String): FeatureRequest<FoldingRangeRequestParams> {
        return request(path) { FoldingRangeRequestParams(it) }
    }

    fun highlight(path: String, line: Int, character: Int): FeatureRequest<TextDocumentPositionParams> {
        return positionRequest(path, line, character) { identifier, position ->
            TextDocumentPositionParams(identifier, position)
        }
    }

    fun hover(path: String, line: Int, character: Int): FeatureRequest<TextDocumentPositionParams> {
        return positionRequest(path, line, character) { identifier, position ->
            TextDocumentPositionParams(identifier, position)
        }
    }

    fun link(path: String): FeatureRequest<DocumentLinkParams> {
        return request(path) { DocumentLinkParams(it) }
    }

    fun reference(path: String, line: Int, character: Int): FeatureRequest<ReferenceParams> {
        return positionRequest(path, line, character) { identifier, position ->
            ReferenceParams().apply {
                textDocument = identifier
                setPosition(position)
            }
        }
    }

    fun rename(path: String, line: Int, character: Int, newName: String): FeatureRequest<RenameParams> {
        return positionRequest(path, line, character) { identifier, position ->
            RenameParams(identifier, position, newName)
        }
    }
}
