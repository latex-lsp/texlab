package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.CompletionItemFactory
import texlab.completion.latex.data.LatexComponentSource
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

class LatexComponentEnvironmentProvider(private val database: LatexComponentSource) : LatexEnvironmentProvider() {
    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: LatexCommandSyntax): List<CompletionItem> {
        return database.getRelatedComponents(request.relatedDocuments)
                .flatMap { component ->
                    component.environments.map {
                        CompletionItemFactory.createEnvironment(it, component.fileNames.joinToString())
                    }
                }
    }
}
