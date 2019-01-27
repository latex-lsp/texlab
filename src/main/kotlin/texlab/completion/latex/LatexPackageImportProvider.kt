package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.resolver.LatexResolver
import texlab.syntax.latex.LatexCommandSyntax

class LatexPackageImportProvider(resolver: LatexResolver) : LatexArgumentProvider() {
    private val items = resolver.filesByName.values
            .filter { it.extension == "sty" }
            .map { CompletionItemFactory.createPackage(it.nameWithoutExtension) }

    override val commandNames = listOf("""\usepackage""")

    override val argumentIndex = 0

    override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return items
    }
}
