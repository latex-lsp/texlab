package texlab

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.*
import org.slf4j.LoggerFactory
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import java.io.File
import java.net.URI

class WorkspaceBuilder {
    companion object {
        private val logger = LoggerFactory.getLogger(::WorkspaceBuilder::class.java)
    }

    var workspace = Workspace()
        private set

    fun document(path: String, text: String): URI {
        val file = File(path)
        val language = getLanguageByExtension(file.extension)!!
        val uri = file.toURI()
        val document = Document.create(uri, text, language)
        workspace = Workspace(workspace.documentsByUri.plus(Pair(document.uri, document)))
        return uri
    }

    fun <R> diagnostics(provider: FeatureProvider<Unit, R>, uri: URI): R {
        return run(provider, uri) { _ -> Unit }
    }

    fun <R> link(provider: FeatureProvider<DocumentLinkParams, R>, uri: URI): R {
        return run(provider, uri, ::DocumentLinkParams)
    }

    fun <R> folding(provider: FeatureProvider<FoldingRangeRequestParams, R>, uri: URI): R {
        return run(provider, uri, ::FoldingRangeRequestParams)
    }

    fun <R> highlight(provider: FeatureProvider<TextDocumentPositionParams, R>,
                      uri: URI,
                      line: Int,
                      character: Int): R {
        return run(provider, uri, line, character, ::TextDocumentPositionParams)
    }

    fun <R> symbol(provider: FeatureProvider<DocumentSymbolParams, R>, uri: URI): R {
        return run(provider, uri, ::DocumentSymbolParams)
    }

    fun <R> completion(provider: FeatureProvider<CompletionParams, R>,
                       uri: URI,
                       line: Int,
                       character: Int): R {
        return run(provider, uri, line, character, ::CompletionParams)
    }

    fun <R> reference(provider: FeatureProvider<ReferenceParams, R>,
                      uri: URI,
                      line: Int,
                      character: Int): R {
        return run(provider, uri, line, character) { identifier, position ->
            ReferenceParams().apply {
                textDocument = identifier
                setPosition(position)
            }
        }
    }

    fun <R> rename(provider: FeatureProvider<RenameParams, R>,
                   uri: URI,
                   line: Int,
                   character: Int,
                   newName: String): R {
        return run(provider, uri, line, character) { identifier, position ->
            RenameParams(identifier, position, newName)
        }
    }

    private fun <T, R> run(provider: FeatureProvider<T, R>,
                           uri: URI,
                           paramsFactory: (identifier: TextDocumentIdentifier) -> T): R {
        return run(provider, uri, 0, 0) { identifier, _ ->
            paramsFactory(identifier)
        }
    }

    private fun <T, R> run(provider: FeatureProvider<T, R>,
                           uri: URI,
                           line: Int,
                           character: Int,
                           paramsFactory: (identifier: TextDocumentIdentifier, Position) -> T): R {
        val identifier = TextDocumentIdentifier(uri.toString())
        val position = Position(line, character)
        val request = FeatureRequest(uri, workspace, paramsFactory(identifier, position), logger)
        return runBlocking {
            provider.get(request)
        }
    }
}