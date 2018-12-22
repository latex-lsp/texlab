package texlab.syntax.latex

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.TextEdit
import texlab.contains

abstract class LatexEnvironmentRenamer {
    companion object {
        fun rename(tree: LatexSyntaxTree, position: Position, newName: String): List<TextEdit>? {
            for (environment in tree.environments) {
                if (environment.beginNameRange.contains(position) || environment.endNameRange.contains(position)) {
                    return listOf(
                            TextEdit(environment.beginNameRange, newName),
                            TextEdit(environment.endNameRange, newName))
                }
            }
            return null
        }
    }
}
