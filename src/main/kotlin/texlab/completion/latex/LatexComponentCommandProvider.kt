package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.completion.latex.data.LatexComponentSource
import texlab.syntax.latex.LatexCommandSyntax

class LatexComponentCommandProvider(private val database: LatexComponentSource) : LatexCommandProvider() {
    override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return database.getRelatedComponents(request.relatedDocuments)
                .flatMap { component ->
                    component.commands.map {
                        CompletionItemFactory.createCommand(it, component.fileNames.joinToString())
                    }
                }
    }
}
