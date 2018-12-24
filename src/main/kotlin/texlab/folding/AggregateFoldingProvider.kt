package texlab.folding

import org.eclipse.lsp4j.FoldingRange

class AggregateFoldingProvider(private vararg val provider: FoldingProvider) : FoldingProvider {
    override fun fold(request: FoldingRequest): List<FoldingRange> {
        return provider.flatMap { it.fold(request) }
    }
}
