package texlab.folding

import org.eclipse.lsp4j.FoldingRange
import org.eclipse.lsp4j.FoldingRangeKind
import org.eclipse.lsp4j.FoldingRangeRequestParams
import texlab.BibtexDocument
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.BibtexDeclarationSyntax

object BibtexDeclarationFoldingProvider : FeatureProvider<FoldingRangeRequestParams, List<FoldingRange>> {
    override suspend fun get(request: FeatureRequest<FoldingRangeRequestParams>): List<FoldingRange> {
        if (request.document !is BibtexDocument) {
            return emptyList()
        }

        return request.document.tree.root
                .children
                .filterIsInstance<BibtexDeclarationSyntax>()
                .mapNotNull { fold(it) }
    }

    private fun fold(entry: BibtexDeclarationSyntax): FoldingRange? {
        val right = entry.right ?: return null
        return FoldingRange().apply {
            startLine = entry.type.line
            startCharacter = entry.type.character
            endLine = right.line
            endCharacter = right.character
            kind = FoldingRangeKind.Region
        }
    }
}
