package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.completion.latex.data.LatexResolver
import texlab.syntax.latex.LatexCommandSyntax

class LatexClassImportProvider(resolver: LatexResolver) : LatexArgumentProvider() {
    private val items = resolver.filesByName.values
            .filter { it.extension == "cls" }
            .map { CompletionItemFactory.createClass(it.nameWithoutExtension) }

    override val commandNames = listOf("""\documentclass""")
    override val argumentIndex = 0

    override fun getItems(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return items
    }
}
