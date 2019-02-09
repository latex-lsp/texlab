package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import texlab.LatexDocument
import texlab.completion.CompletionProvider
import texlab.completion.CompletionRequest
import texlab.contains
import texlab.syntax.latex.LatexCommandSyntax

abstract class LatexCommandProvider : CompletionProvider {
    override fun complete(request: CompletionRequest): List<CompletionItem> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        val command = request.document.tree.root
                .descendants()
                .filterIsInstance<LatexCommandSyntax>()
                .lastOrNull { getCompletionRange(it).contains(request.position) }

        return if (command is LatexCommandSyntax) {
            complete(request, command)
        } else {
            emptyList()
        }
    }

    private fun getCompletionRange(command: LatexCommandSyntax): Range {
        val start = Position(command.name.line, command.name.character + 1)
        val end = command.name.end
        return Range(start, end)
    }

    protected abstract fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem>
}
