package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.LatexDocument
import texlab.completion.CompletionItemFactory
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexUserCommandProvider : LatexCommandProvider() {
    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: LatexCommandSyntax): List<CompletionItem> {
        return request.relatedDocuments
                .filterIsInstance<LatexDocument>()
                .flatMap { it.tree.root.descendants() }
                .filterIsInstance<LatexCommandSyntax>()
                .minus(command)
                .map { it.name.text.substring(1) }
                .distinct()
                .map { CompletionItemFactory.createCommand(it, "unknown") }
    }
}
