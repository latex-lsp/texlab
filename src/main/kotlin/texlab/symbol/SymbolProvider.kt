package texlab.symbol

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.DocumentSymbolParams
import texlab.provider.FeatureProvider

object SymbolProvider : FeatureProvider<DocumentSymbolParams, List<DocumentSymbol>> by
FeatureProvider.concat(
        LatexCommandSymbolProvider,
        LatexEnvironmentSymbolProvider,
        LatexLabelSymbolProvider,
        LatexCitationSymbolProvider,
        BibtexEntrySymbolProvider)
