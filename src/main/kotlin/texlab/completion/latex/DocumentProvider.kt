package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.Document
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.latex.LatexCommandSyntax
import java.net.URI
import java.nio.file.Paths

abstract class DocumentProvider<T>(private val documentClass: Class<T>,
                                   private val includeExtension: Boolean)
    : LatexArgumentProvider() where T : Document {
    override val argumentIndex: Int = 0

    override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return request.workspace.documents
                .filterIsInstance(documentClass)
                .filter { !request.relatedDocuments.contains(it) }
                .map { relativize(request.uri, it.uri) }
                .map { CompletionItemFactory.createFile(it) }
    }

    private fun relativize(base: URI, relative: URI): String {
        val pathAbsolute = Paths.get(relative)
        var path = Paths.get(base)
                .parent
                .relativize(pathAbsolute)
                .toString()
                .replace('\\', '/')

        if (!includeExtension && path.contains('.')) {
            path = path.substring(0, path.lastIndexOf('.'))
        }

        return path
    }
}
