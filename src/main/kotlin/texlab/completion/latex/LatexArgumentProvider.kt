package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.LatexDocument
import texlab.completion.CompletionProvider
import texlab.completion.CompletionRequest
import texlab.contains
import texlab.syntax.latex.LatexCommandSyntax
import texlab.syntax.latex.LatexGroupSyntax
import texlab.syntax.latex.LatexSyntaxNode
import texlab.syntax.latex.LatexTextSyntax

abstract class LatexArgumentProvider : CompletionProvider {

    abstract val commandNames: List<String>

    abstract val argumentIndex: Int

    override fun getItems(request: CompletionRequest): List<CompletionItem> {
        return if (request.document is LatexDocument) {
            val nodes = request.document
                    .tree
                    .root
                    .descendants()
                    .filter { it.range.contains(request.position) }
                    .asReversed()

            val command = findNonEmptyCommand(nodes) ?: findEmptyCommand(nodes)
            if (command == null) {
                listOf()
            } else {
                getItems(request, command)
            }
        } else {
            listOf()
        }
    }

    private fun findNonEmptyCommand(nodes: List<LatexSyntaxNode>): LatexCommandSyntax? {
        return if (nodes.size >= 3 && nodes[0] is LatexTextSyntax) {
            findCommand(nodes, 1)
        } else {
            null
        }
    }

    private fun findEmptyCommand(nodes: List<LatexSyntaxNode>): LatexCommandSyntax? {
        return if (nodes.size >= 2) {
            findCommand(nodes, 0)
        } else {
            null
        }
    }

    private fun findCommand(nodes: List<LatexSyntaxNode>, index: Int): LatexCommandSyntax? {
        if (nodes[index] is LatexGroupSyntax && nodes[index + 1] is LatexCommandSyntax) {
            val group = nodes[index] as LatexGroupSyntax
            val command = nodes[index + 1] as LatexCommandSyntax
            if (commandNames.contains(command.name.text) &&
                    command.args.size > argumentIndex && command.args[argumentIndex] == group) {
                return command
            }
        }
        return null
    }

    protected abstract fun getItems(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem>
}
