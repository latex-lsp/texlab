package texlab.folding

import org.eclipse.lsp4j.FoldingRange
import org.eclipse.lsp4j.FoldingRangeKind
import org.eclipse.lsp4j.FoldingRangeRequestParams
import texlab.LatexDocument
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest

object LatexEnvironmentFoldingProvider : FeatureProvider<FoldingRangeRequestParams, List<FoldingRange>> {
    override suspend fun get(request: FeatureRequest<FoldingRangeRequestParams>): List<FoldingRange> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val foldings = mutableListOf<FoldingRange>()
        for (environment in request.document.tree.environments) {
            foldings.add(FoldingRange().apply {
                startLine = environment.begin.end.line
                startCharacter = environment.begin.end.character
                endLine = environment.end.start.line
                endCharacter = environment.end.start.character
                kind = FoldingRangeKind.Region
            })
        }
        return foldings
    }
}
