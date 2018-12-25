package texlab

import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.eclipse.lsp4j.services.TextDocumentService
import texlab.completion.AggregateCompletionProvider
import texlab.completion.CompletionProvider
import texlab.completion.CompletionRequest
import texlab.completion.OrderByQualityProvider
import texlab.completion.latex.*
import texlab.completion.latex.data.LatexResolver
import texlab.folding.*
import texlab.link.AggregateLinkProvider
import texlab.link.LatexIncludeLinkProvider
import texlab.link.LinkProvider
import texlab.link.LinkRequest
import texlab.rename.*
import texlab.symbol.*
import java.net.URI
import java.util.concurrent.CompletableFuture

class TextDocumentServiceImpl(private val workspace: Workspace) : TextDocumentService {
    private val resolver = LatexResolver.create()

    private val completionProvider: CompletionProvider =
            OrderByQualityProvider(
                    AggregateCompletionProvider(
                            LatexIncludeProvider(workspace),
                            LatexBibliographyProvider(workspace),
                            LatexPackageImportProvider(resolver),
                            LatexClassImportProvider(resolver),
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

    private val symbolProvider: SymbolProvider =
            AggregateSymbolProvider(
                    LatexEnvironmentSymbolProvider,
                    LatexLabelSymbolProvider)

    private val renamer: Renamer =
            AggregateRenamer(
                    LatexEnvironmentRenamer,
                    LatexLabelRenamer)

    private val foldingProvider: FoldingProvider =
            AggregateFoldingProvider(
                    LatexEnvironmentFoldingProvider,
                    LatexSectionFoldingProvider)

    private val linkProvider: LinkProvider = AggregateLinkProvider(LatexIncludeLinkProvider)

    companion object {
        private const val MAX_COMPLETIONS_ITEMS_COUNT = 100
    }

    override fun didOpen(params: DidOpenTextDocumentParams) {
        params.textDocument.apply {
            val language = getLanguageById(languageId) ?: return
            synchronized(workspace) {
                val document = Document.create(URI.create(uri), language)
                document.text = text
                document.version = version
                document.analyze()
                workspace.documents.add(document)
            }
        }
    }

    override fun didChange(params: DidChangeTextDocumentParams) {
        val uri = URI.create(params.textDocument.uri)
        synchronized(workspace) {
            val document = workspace.documents.first { it.uri == uri }
            if (document.version <= params.textDocument.version) {
                params.contentChanges.forEach {
                    document.text = it.text
                }
                document.version = params.textDocument.version
                document.analyze()
            }
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
            val document = workspace.documents
                    .firstOrNull { it.uri == uri }
                    ?: return CompletableFuture.completedFuture(null)

            val request = SymbolRequest(document)
            val symbols = symbolProvider
                    .getSymbols(request)
                    .map { Either.forRight<SymbolInformation, DocumentSymbol>(it) }
                    .toMutableList()
            return CompletableFuture.completedFuture(symbols)
        }
    }

    override fun rename(params: RenameParams): CompletableFuture<WorkspaceEdit?> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = RenameRequest(uri, relatedDocuments, params.position, params.newName)
            return CompletableFuture.completedFuture(renamer.rename(request))
        }
    }

    override fun documentLink(params: DocumentLinkParams): CompletableFuture<MutableList<DocumentLink>> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val request = LinkRequest(uri, workspace)
            val links = linkProvider.getLinks(request).toMutableList()
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

            val request = FoldingRequest(document)
            val foldings = foldingProvider.fold(request).toMutableList()
            return CompletableFuture.completedFuture(foldings)
        }
    }
}
