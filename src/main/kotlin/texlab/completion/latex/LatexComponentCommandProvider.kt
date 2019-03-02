package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.CompletionItemFactory
import texlab.completion.latex.data.LatexComponentSource
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

class LatexComponentCommandProvider(private val database: LatexComponentSource) : LatexCommandProvider() {
    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: LatexCommandSyntax): List<CompletionItem> {
        return database.getRelatedComponents(request.relatedDocuments)
                .flatMap { component ->
                    component.commands.map {
                        CompletionItemFactory.createCommand(it, component.fileNames.joinToString())
                    }
                }
    }
}
