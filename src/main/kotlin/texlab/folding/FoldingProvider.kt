package texlab.folding

import org.eclipse.lsp4j.FoldingRange
import org.eclipse.lsp4j.FoldingRangeRequestParams
import texlab.provider.FeatureProvider

object FoldingProvider : FeatureProvider<FoldingRangeRequestParams, List<FoldingRange>> by
FeatureProvider.concat(
        LatexEnvironmentFoldingProvider,
        LatexSectionFoldingProvider,
        BibtexDeclarationFoldingProvider)
