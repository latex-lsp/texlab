package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.LatexDocument
import texlab.completion.CompletionItemFactory
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax
import texlab.syntax.latex.LatexLabel

object LatexLabelProvider : LatexArgumentProvider() {
    override val commandNames: List<String> = LatexLabel.REFERENCE_COMMANDS

    override val argumentIndex: Int = 0

    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: LatexCommandSyntax): List<CompletionItem> {
        return request.relatedDocuments
                .filterIsInstance<LatexDocument>()
                .flatMap { it.tree.labelDefinitions }
                .map { it.name.text }
                .distinct()
                .map { CompletionItemFactory.createLabel(it) }
    }
}
