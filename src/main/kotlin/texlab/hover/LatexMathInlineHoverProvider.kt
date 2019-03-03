package texlab.hover

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import texlab.LatexDocument
import texlab.contains

object LatexMathInlineHoverProvider : LatexMathHoverProvider() {
    override fun getCodeRange(document: LatexDocument, position: Position): Range? {
        return document.tree.inlines
                .map { it.range }
                .firstOrNull { it.contains(position) }
    }
}
