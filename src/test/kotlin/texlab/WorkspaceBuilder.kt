package texlab

import org.eclipse.lsp4j.*
import org.slf4j.LoggerFactory
import texlab.provider.FeatureRequest
import java.io.File
import java.net.URI

class WorkspaceBuilder {
    companion object {
        private val logger = LoggerFactory.getLogger(::WorkspaceBuilder::class.java)
    }

    var workspace = Workspace()

    fun uri(path: String): URI {
        val file = File(path)
        return URIHelper.parse(file.toURI().toString())
    }

    fun document(uri: URI, text: String): WorkspaceBuilder {
        val file = File(uri)
        val language = getLanguageByExtension(file.extension)!!
        val document = Document.create(uri, text, language)
        workspace = Workspace(workspace.documentsByUri.plus(Pair(document.uri, document)))
        return this
    }

    fun document(path: String, text: String): WorkspaceBuilder {
        return document(uri(path), text)
    }

    fun <T> request(path: String, paramsFactory: (TextDocumentIdentifier) -> T): FeatureRequest<T> {
        val uri = uri(path)
        val identifier = TextDocumentIdentifier(uri.toString())
        val params = paramsFactory(identifier)
        return FeatureRequest(uri, workspace, params, logger)
    }

    private fun <T> positionRequest(path: String,
                                    line: Int,
                                    character: Int,
                                    paramsFactory: (TextDocumentIdentifier, Position) -> T): FeatureRequest<T> {
        val uri = uri(path)
        val position = Position(line, character)
        val identifier = TextDocumentIdentifier(uri.toString())
        val params = paramsFactory(identifier, position)
        return FeatureRequest(uri, workspace, params, logger)
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
