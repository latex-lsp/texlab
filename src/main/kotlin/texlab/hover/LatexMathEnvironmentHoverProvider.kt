package texlab.hover

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import texlab.LatexDocument
import texlab.contains


object LatexMathEnvironmentHoverProvider : LatexMathHoverProvider() {
    override fun getCodeRange(document: LatexDocument, position: Position): Range? {
        val environment = document.tree
                .environments
                .firstOrNull { it.range.contains(position) }
                ?: return null

        val name = environment.beginName.replace("*", "")
        return if (LatexFormulaRenderer.ENVIRONMENTS.contains(name)) {
            environment.range
        } else {
            null
        }
    }
}
