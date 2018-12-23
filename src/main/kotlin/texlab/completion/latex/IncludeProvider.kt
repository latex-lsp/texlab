package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.Document
import texlab.Workspace
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.latex.LatexCommandSyntax
import java.net.URI
import java.nio.file.Paths

abstract class IncludeProvider<T>(private val workspace: Workspace,
                                  private val documentClass: Class<T>) : LatexArgumentProvider() where T : Document {

    override val argumentIndex: Int = 0

    override fun getItems(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return workspace.documents
                .filterIsInstance(documentClass)
                .filter { !request.relatedDocuments.contains(it) }
                .map { relativize(request.uri, it.uri) }
                .map { CompletionItemFactory.createFile(it) }
    }

    private fun relativize(base: URI, relative: URI): String {
        val pathAbsolute = Paths.get(relative)
        val pathBase = Paths.get(base).parent
        return pathBase.relativize(pathAbsolute).toString().replace('\\', '/')
    }
}
