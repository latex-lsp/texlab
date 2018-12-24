package texlab

import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.eclipse.lsp4j.services.TextDocumentService
import texlab.completion.AggregateProvider
import texlab.completion.CompletionProvider
import texlab.completion.CompletionRequest
import texlab.completion.OrderByQualityProvider
import texlab.completion.latex.*
import java.net.URI
import java.util.concurrent.CompletableFuture

class TextDocumentServiceImpl(private val workspace: Workspace) : TextDocumentService {

    private val completionProvider: CompletionProvider =
            OrderByQualityProvider(
                    AggregateProvider(
                            LatexIncludeProvider(workspace),
                            LatexBibliographyProvider(workspace),
                            PgfLibraryProvider(),
                            TikzLibraryProvider(),
                            LatexColorProvider(),
                            DefineColorModelProvider(),
                            DefineColorSetModelProvider(),
                            LatexLabelProvider(),
                            LatexBeginCommandProvider(),
                            LatexKernelEnvironmentProvider(),
                            LatexUserEnvironmentProvider(),
                            LatexKernelCommandProvider(),
                            LatexUserCommandProvider()))

    companion object {
        private const val MAX_COMPLETIONS_ITEMS_COUNT = 100
    }

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
                    ?.documentSymbol(workspace)
                    ?.map { Either.forRight<SymbolInformation, DocumentSymbol>(it) }
                    ?.toMutableList()
                    ?: mutableListOf()
            return CompletableFuture.completedFuture(symbols)
        }
    }

    override fun rename(params: RenameParams): CompletableFuture<WorkspaceEdit?> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val document = workspace.documents
                    .firstOrNull { it.uri == uri }
                    ?: return CompletableFuture.completedFuture(null)

            val edit = document.rename(workspace, params.position, params.newName)
            return CompletableFuture.completedFuture(edit)
        }
    }

    override fun documentLink(params: DocumentLinkParams): CompletableFuture<MutableList<DocumentLink>> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val links = workspace.documents
                    .filter { it.isFile }
                    .firstOrNull { it.uri == uri }
                    ?.documentLink(workspace)
                    ?.toMutableList()
                    ?: mutableListOf()

            return CompletableFuture.completedFuture(links)
        }
    }

    override fun completion(params: CompletionParams):
            CompletableFuture<Either<MutableList<CompletionItem>, CompletionList>> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = CompletionRequest(uri, relatedDocuments, params.position)
            val items = completionProvider.getItems(request).toList()
            val list = CompletionList(items.size == MAX_COMPLETIONS_ITEMS_COUNT, items)
            return CompletableFuture.completedFuture(Either.forRight(list))
        }
    }

    override fun foldingRange(params: FoldingRangeRequestParams): CompletableFuture<MutableList<FoldingRange>> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val document = workspace.documents
                    .firstOrNull { it.uri == uri }
                    ?: return CompletableFuture.completedFuture(null)

            val foldings = document.foldingRange().toMutableList()
            return CompletableFuture.completedFuture(foldings)
        }
    }
}

