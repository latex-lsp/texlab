package texlab.hover

import kotlinx.coroutines.Deferred
import kotlinx.coroutines.ObsoleteCoroutinesApi
import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.completion.latex.data.LatexComponentDatabase
import texlab.provider.DeferredProvider
import texlab.provider.FeatureProvider

@ObsoleteCoroutinesApi
class HoverProvider(componentDatabase: Deferred<LatexComponentDatabase>)
    : FeatureProvider<TextDocumentPositionParams, Hover?> by FeatureProvider.choice(
        LatexComponentHoverProvider,
        LatexCitationHoverProvider,
        LatexMathEnvironmentHoverProvider,
        LatexMathEquationHoverProvider,
        LatexMathInlineHoverProvider,
        DeferredProvider(::LatexCommandHoverProvider, componentDatabase, null),
        BibtexEntryTypeHoverProvider,
        BibtexFieldHoverProvider)
