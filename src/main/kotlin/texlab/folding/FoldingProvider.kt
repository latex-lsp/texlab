package texlab.folding

import org.eclipse.lsp4j.FoldingRange

interface FoldingProvider {
    fun fold(request: FoldingRequest): List<FoldingRange>
}

