package texlab.completion

import kotlinx.coroutines.Deferred
import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.LatexLanguageServerConfig
import texlab.completion.bibtex.BibtexEntryTypeProvider
import texlab.completion.bibtex.BibtexFieldNameProvider
import texlab.completion.bibtex.BibtexKernelCommandProvider
import texlab.completion.latex.*
import texlab.completion.latex.data.LatexComponentDatabase
import texlab.completion.latex.data.symbols.LatexArgumentSymbolProvider
import texlab.completion.latex.data.symbols.LatexCommandSymbolProvider
import texlab.completion.latex.data.symbols.LatexSymbolDatabase
import texlab.provider.DeferredProvider
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.resolver.LatexResolver

class CompletionProvider(resolver: Deferred<LatexResolver>,
                         componentDatabase: Deferred<LatexComponentDatabase>,
                         symbolDatabase: Deferred<LatexSymbolDatabase>)
    : FeatureProvider<CompletionParams, List<CompletionItem>> {
    private val provider = FeatureProvider.concat(
            LatexIncludeProvider,
            DeferredProvider(::LatexClassImportProvider, resolver, emptyList()),
            DeferredProvider(::LatexPackageImportProvider, resolver, emptyList()),
            PgfLibraryProvider,
            TikzLibraryProvider,
            LatexCitationProvider,
            LatexColorProvider,
            DefineColorModelProvider,
            DefineColorSetModelProvider,
            LatexLabelProvider,
            LatexBeginCommandProvider,
            DeferredProvider(::LatexComponentEnvironmentProvider, componentDatabase, emptyList()),
            LatexKernelEnvironmentProvider,
            LatexUserEnvironmentProvider,
            DeferredProvider(::LatexArgumentSymbolProvider, symbolDatabase, emptyList()),
            DeferredProvider(::LatexCommandSymbolProvider, symbolDatabase, emptyList()),
            DeferredProvider(::TikzCommandProvider, componentDatabase, emptyList()),
            DeferredProvider(::LatexComponentCommandProvider, componentDatabase, emptyList()),
            LatexKernelCommandProvider,
            LatexUserCommandProvider,
            BibtexEntryTypeProvider,
            BibtexFieldNameProvider,
            BibtexKernelCommandProvider)

    override suspend fun get(request: FeatureRequest<CompletionParams>): List<CompletionItem> {
        val qualityEvaluator = MatchQualityEvaluator(request.document, request.params.position)
        return provider.get(request)
                .distinctBy { it.label }
                .sortedByDescending { qualityEvaluator.evaluate(it) }
                .take(LatexLanguageServerConfig.COMPLETION_LIMIT)
    }
}
