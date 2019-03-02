package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.CompletionItemFactory
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

abstract class LatexColorModelProvider : LatexArgumentProvider() {
    private val models: Array<String> = arrayOf("gray", "rgb", "RGB", "HTML", "cmyk")

    private val items: List<CompletionItem> = models.map { CompletionItemFactory.createColorModel(it) }

    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: LatexCommandSyntax): List<CompletionItem> {
        return items
    }
}

