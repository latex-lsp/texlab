package texlab

import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.eclipse.lsp4j.services.TextDocumentService
import texlab.completion.*
import texlab.completion.bibtex.BibtexEntryTypeProvider
import texlab.completion.bibtex.BibtexFieldNameProvider
import texlab.completion.bibtex.BibtexKernelCommandProvider
import texlab.completion.latex.*
import texlab.completion.latex.data.LatexComponentDatabase
import texlab.completion.latex.data.LatexComponentDatabaseListener
import texlab.completion.latex.data.LatexResolver
import texlab.definition.*
import texlab.folding.*
import texlab.link.AggregateLinkProvider
import texlab.link.LatexIncludeLinkProvider
import texlab.link.LinkProvider
import texlab.link.LinkRequest
import texlab.metadata.CtanPackageMetadataProvider
import texlab.metadata.PackageMetadataProvider
import texlab.rename.*
import texlab.symbol.*
import java.io.File
import java.net.URI
import java.nio.file.Paths
import java.util.concurrent.CompletableFuture


class TextDocumentServiceImpl(private val workspace: Workspace) : TextDocumentService {
    lateinit var client: LanguageClientExtensions
    private val resolver = LatexResolver.create()

    private val databaseDirectory = Paths.get(javaClass.protectionDomain.codeSource.location.toURI()).parent
    private val databaseFile = databaseDirectory.resolve("components.json")
    private val database =
            LatexComponentDatabase.loadOrCreate(databaseFile, resolver, object : LatexComponentDatabaseListener {
                override fun onStartIndexing(file: File) {
                    client.setStatus(StatusParams(ServerStatus.INDEXING, file.name))
                }

                override fun onStopIndexing() {
                    client.setStatus(StatusParams(ServerStatus.IDLE))
                }
            })

    private val completionProvider: CompletionProvider =
            LimitedCompletionProvider(
                    OrderByQualityProvider(
                            AggregateCompletionProvider(
                                    LatexIncludeProvider(workspace),
                                    LatexBibliographyProvider(workspace),
                                    LatexClassImportProvider(resolver),
                                    LatexPackageImportProvider(resolver),
                                    PgfLibraryProvider,
                                    TikzLibraryProvider,
                                    LatexCitationProvider,
                                    LatexColorProvider,
                                    DefineColorModelProvider,
                                    DefineColorSetModelProvider,
                                    LatexLabelProvider,
                                    LatexBeginCommandProvider,
                                    LatexComponentEnvironmentProvider(database),
                                    LatexKernelEnvironmentProvider,
                                    LatexUserEnvironmentProvider,
                                    LatexComponentCommandProvider(database),
                                    LatexKernelCommandProvider,
                                    LatexUserCommandProvider,
                                    BibtexEntryTypeProvider,
                                    BibtexFieldNameProvider,
                                    BibtexKernelCommandProvider)))

    private val symbolProvider: SymbolProvider =
            AggregateSymbolProvider(
                    LatexCommandSymbolProvider,
                    LatexEnvironmentSymbolProvider,
                    LatexLabelSymbolProvider,
                    LatexCitationSymbolProvider,
                    BibtexEntrySymbolProvider)

    private val renamer: Renamer =
            AggregateRenamer(
                    LatexCommandRenamer,
                    LatexEnvironmentRenamer,
                    LatexLabelRenamer,
                    BibtexEntryRenamer)

    private val foldingProvider: FoldingProvider =
            AggregateFoldingProvider(
                    LatexEnvironmentFoldingProvider,
                    LatexSectionFoldingProvider,
                    BibtexEntryFoldingProvider)

    private val linkProvider: LinkProvider = AggregateLinkProvider(LatexIncludeLinkProvider)

    private val definitionProvider: DefinitionProvider =
            AggregateDefinitionProvider(
                    LatexLabelDefinitionProvider,
                    BibtexEntryDefinitionProvider)

    private val metadataProvider: PackageMetadataProvider = CtanPackageMetadataProvider()

    override fun didOpen(params: DidOpenTextDocumentParams) {
        params.textDocument.apply {
            val language = getLanguageById(languageId) ?: return
            synchronized(workspace) {
                val uri = URI.create(uri)
                val document = workspace.documents.firstOrNull { it.uri == uri } ?: Document.create(uri, language)
                document.text = text
                document.version = version
                document.analyze()
                if (!workspace.documents.contains(document)) {
                    workspace.documents.add(document)
                }
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
            val items = completionProvider.complete(request).toList()
            val list = CompletionList(true, items)
            return CompletableFuture.completedFuture(Either.forRight(list))
        }
    }

    override fun resolveCompletionItem(unresolved: CompletionItem): CompletableFuture<CompletionItem> {
        return CompletableFuture.supplyAsync<CompletionItem> {
            if (unresolved.kind == CompletionItemKind.Class) {
                val metadata = metadataProvider.getMetadata(unresolved.label)
                if (metadata != null) {
                    unresolved.detail = metadata.caption
                    unresolved.setDocumentation(metadata.description)
                }
            }

            unresolved
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

    override fun definition(params: TextDocumentPositionParams): CompletableFuture<MutableList<out Location>> {
        synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val relatedDocuments = workspace.relatedDocuments(uri)
            val request = DefinitionRequest(uri, relatedDocuments, params.position)
            val location = definitionProvider.find(request)
            return CompletableFuture.completedFuture(location?.let { mutableListOf(it) })
        }
    }

    override fun hover(params: TextDocumentPositionParams): CompletableFuture<Hover> {
        val include = synchronized(workspace) {
            val uri = URI.create(params.textDocument.uri)
            val document = workspace.documents
                    .filterIsInstance<LatexDocument>()
                    .firstOrNull { it.uri == uri }
                    ?: return CompletableFuture.completedFuture(null)

            document.tree.includes
                    .filter { it.command.name.text == "\\usepackage" || it.command.name.text == "\\documentclass" }
                    .firstOrNull { it.command.range.contains(params.position) }
                    ?: return CompletableFuture.completedFuture(null)
        }

        return CompletableFuture.supplyAsync {
            val metadata = metadataProvider.getMetadata(include.path)
            val description = metadata?.description
            if (description != null) {
                Hover(description)
            } else {
                null
            }
        }
    }
}
