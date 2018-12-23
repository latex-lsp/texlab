package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.LatexDocument
import texlab.completion.CompletionProvider
import texlab.completion.CompletionRequest
import texlab.contains
import texlab.syntax.latex.LatexCommandSyntax

abstract class LatexCommandProvider : CompletionProvider {

    override fun getItems(request: CompletionRequest): Sequence<CompletionItem> {
        return if (request.document is LatexDocument) {
            val command = request.document
                    .tree
                    .root
                    .descendants()
                    .lastOrNull { it.range.contains(request.position) }

            if (command is LatexCommandSyntax) {
                getItems(request, command)
            } else {
                sequenceOf()
            }
        } else {
            sequenceOf()
        }
    }

    protected abstract fun getItems(request: CompletionRequest, command: LatexCommandSyntax): Sequence<CompletionItem>
}

//public abstract class LatexCommandProvider : ICompletionProvider
//{
//    public IEnumerable<CompletionItem> GetItems(CompletionRequest request)
//    {
//        return request.Nodes.Count > 0 && request.Nodes[0] is LatexCommandSyntax command
//        ? GetItems(request, command)
//        : Enumerable.Empty<CompletionItem>();
//    }
//
//    protected abstract IEnumerable<CompletionItem> GetItems(CompletionRequest request, LatexCommandSyntax command);
//}
