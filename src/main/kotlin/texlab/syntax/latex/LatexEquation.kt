package texlab.syntax.latex

import org.eclipse.lsp4j.Range

data class LatexEquation(val begin: LatexCommandSyntax, val end: LatexCommandSyntax) {
    val range: Range
        get() = Range(begin.start, end.end)

    companion object {
        fun find(root: LatexSyntaxNode): List<LatexEquation> {
            val equations = mutableListOf<LatexEquation>()
            var begin: LatexCommandSyntax? = null
            for (command in root.descendants().filterIsInstance<LatexCommandSyntax>()) {
                if (command.name.text == "\\[") {
                    begin = command
                } else if (command.name.text == "\\]" && begin != null) {
                    equations.add(LatexEquation(begin, command))
                }
            }
            return equations
        }
    }
}
