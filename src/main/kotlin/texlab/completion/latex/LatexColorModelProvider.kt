package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.latex.LatexCommandSyntax

abstract class LatexColorModelProvider : LatexArgumentProvider() {

    private val models: Array<String> = arrayOf("gray", "rgb", "RGB", "HTML", "cmyk")

    private val items: List<CompletionItem> = models.map { CompletionItemFactory.createColorModel(it) }

    override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return items
    }
}

