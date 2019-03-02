package texlab.completion.bibtex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.BibtexDocument
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.BibtexCommandSyntax

abstract class BibtexCommandProvider : FeatureProvider<CompletionParams, CompletionItem> {
    override suspend fun get(request: FeatureRequest<CompletionParams>): List<CompletionItem> {
        if (request.document !is BibtexDocument || request.params.position == null) {
            return emptyList()
        }

        val command = request.document.tree.root
                .descendants()
                .lastOrNull { it.range.contains(request.params.position) }

        return if (command is BibtexCommandSyntax) {
            complete(request, command)
        } else {
            emptyList()
        }
    }

    protected abstract fun complete(request: FeatureRequest<CompletionParams>,
                                    command: BibtexCommandSyntax): List<CompletionItem>
}
