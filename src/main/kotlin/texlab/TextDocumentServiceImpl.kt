package texlab

import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.eclipse.lsp4j.services.TextDocumentService
import java.net.URI
import java.util.concurrent.CompletableFuture

class TextDocumentServiceImpl(private val workspace: Workspace) : TextDocumentService {

    override fun didOpen(params: DidOpenTextDocumentParams) {
        params.textDocument.apply {
            val language = getLanguageById(languageId) ?: return
            synchronized(workspace) {
                workspace.create(URI.create(uri), language, text)
            }
        }
    }

    override fun didChange(params: DidChangeTextDocumentParams) {
        val uri = URI.create(params.textDocument.uri)
        synchronized(workspace) {
            workspace.update(uri, params.contentChanges, params.textDocument.version)
        }
    }

    override fun didSave(params: DidSaveTextDocumentParams) {
    }

    override fun didClose(params: DidCloseTextDocumentParams) {
    }

    override fun documentSymbol(params: DocumentSymbolParams):
            CompletableFuture<MutableList<Either<SymbolInformation, DocumentSymbol>>> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val symbols = workspace.documents
                    .firstOrNull { it.uri == uri }
                    ?.documentSymbol()
                    ?.map { Either.forRight<SymbolInformation, DocumentSymbol>(it) }
                    ?.toMutableList()
                    ?: mutableListOf()
            return CompletableFuture.completedFuture(symbols)
        }
    }

    override fun rename(params: RenameParams): CompletableFuture<WorkspaceEdit?> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val documents = workspace.relatedDocuments(uri)
            if (documents.isEmpty()) {
                return CompletableFuture.completedFuture(null)
            }

            val edit = documents[0].rename(documents, params.position, params.newName)
            return CompletableFuture.completedFuture(edit)
        }
    }
}

