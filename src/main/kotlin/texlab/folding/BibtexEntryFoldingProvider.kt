package texlab.folding

import org.eclipse.lsp4j.FoldingRange
import org.eclipse.lsp4j.FoldingRangeKind
import texlab.BibtexDocument
import texlab.syntax.bibtex.BibtexEntrySyntax

object BibtexEntryFoldingProvider : FoldingProvider {
    override fun fold(request: FoldingRequest): List<FoldingRange> {
        if (request.document !is BibtexDocument) {
            return emptyList()
        }

        return request.document.tree.root
                .children
                .filterIsInstance<BibtexEntrySyntax>()
                .mapNotNull { fold(it) }
    }

    private fun fold(entry: BibtexEntrySyntax): FoldingRange? {
        if (entry.right == null) {
            return null
        }

        return FoldingRange().apply {
            startLine = entry.type.line
            startCharacter = entry.type.character
            endLine = entry.right.line
            endCharacter = entry.right.character
            kind = FoldingRangeKind.Region
        }
    }
}
