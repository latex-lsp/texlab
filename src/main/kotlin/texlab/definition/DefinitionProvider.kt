package texlab.definition

import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.TextDocumentPositionParams
import texlab.provider.FeatureProvider

object DefinitionProvider : FeatureProvider<TextDocumentPositionParams, List<Location>>
by FeatureProvider.concat(
        LatexLabelDefinitionProvider,
        BibtexEntryDefinitionProvider)
