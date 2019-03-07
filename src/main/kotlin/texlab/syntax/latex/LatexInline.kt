package texlab.syntax.latex

import org.eclipse.lsp4j.Range

data class LatexInline(val begin: LatexToken, val end: LatexToken) {
    val range: Range
        get() = Range(begin.start, end.end)

    companion object {
        fun find(root: LatexSyntaxNode): List<LatexInline> {
            val inlines = mutableListOf<LatexInline>()
            var begin: LatexToken? = null
            for (text in root.descendants.filterIsInstance<LatexTextSyntax>()) {
                for (math in text.words.filter { it.kind == LatexTokenKind.MATH }) {
                    if (begin == null) {
                        begin = math
                    } else {
                        inlines.add(LatexInline(begin, math))
                        begin = null
                    }
                }
            }
            return inlines
        }
    }
}
