package texlab.syntax.latex

import org.eclipse.lsp4j.Range
import java.util.*

data class LatexEnvironment(val begin: LatexCommandSyntax, val end: LatexCommandSyntax) {
    val beginName: String
        get() = begin.extractWord(0) ?: ""

    val endName: String
        get() = end.extractWord(0) ?: ""

    val beginNameRange: Range
        get() = getNameRange(begin)

    val endNameRange: Range
        get() = getNameRange(end)

    private fun getNameRange(delimiter: LatexCommandSyntax): Range {
        val group = delimiter.args[0]
        return if (group.children.isNotEmpty()) {
            group.children[0].range
        } else {
            Range(group.left.end, group.left.end)
        }
    }

    companion object {
        private val COMMAND_NAMES = arrayOf("\\begin", "\\end")

        fun analyze(root: LatexSyntaxNode): List<LatexEnvironment> {
            val environments = mutableListOf<LatexEnvironment>()
            val stack = Stack<LatexEnvironmentDelimiter>()
            root.descendants()
                    .filterIsInstance<LatexCommandSyntax>()
                    .filter { COMMAND_NAMES.contains(it.name.text) }
                    .mapNotNull { analyze(it) }
                    .forEach { delimiter ->
                        if (delimiter.kind == LatexEnvironmentDelimiterKind.BEGIN) {
                            stack.push(delimiter)
                        } else {
                            if (!stack.isEmpty()) {
                                environments.add(LatexEnvironment(stack.pop().command, delimiter.command))
                            }
                        }
                    }
            return environments
        }

        private fun analyze(command: LatexCommandSyntax): LatexEnvironmentDelimiter? {
            val kind = if (command.name.text == COMMAND_NAMES[0]) {
                LatexEnvironmentDelimiterKind.BEGIN
            } else {
                LatexEnvironmentDelimiterKind.END
            }

            if (command.args.isNotEmpty() && command.args[0].children.isEmpty()) {
                return LatexEnvironmentDelimiter(command, kind)
            }

            val name = command.extractWord(0)
            return if (name == null) {
                null
            } else {
                LatexEnvironmentDelimiter(command, kind)
            }
        }
    }
}
