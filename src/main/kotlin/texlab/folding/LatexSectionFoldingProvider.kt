package texlab.folding

import org.eclipse.lsp4j.FoldingRange
import org.eclipse.lsp4j.FoldingRangeKind
import texlab.LatexDocument
import texlab.syntax.latex.LatexSection

object LatexSectionFoldingProvider : FoldingProvider {
    override fun fold(request: FoldingRequest): List<FoldingRange> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val foldings = mutableListOf<FoldingRange>()
        val sections = request.document.tree.sections
        for (i in 0 until sections.size) {
            val current = sections[i]
            var next: LatexSection? = null
            for (j in i + 1 until sections.size) {
                next = sections[j]
                if (current.level >= sections[j].level) {
                    break
                }
            }

            if (next != null) {
                val folding = FoldingRange().apply {
                    startLine = current.command.range.end.line
                    startCharacter = current.command.range.end.character
                    endLine = next.command.range.start.line - 1
                    endCharacter = 0
                    kind = FoldingRangeKind.Region
                }
                foldings.add(folding)
            }
        }
        return foldings
    }
}
