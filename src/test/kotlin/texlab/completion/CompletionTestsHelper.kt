package texlab.completion

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.Position
import texlab.Language
import texlab.Workspace
import java.net.URI

object CompletionTestsHelper {

    fun createProvider(vararg items: String): CompletionProvider {
        return object : CompletionProvider {
            override fun getItems(request: CompletionRequest): List<CompletionItem> {
                return items.map { CompletionItem(it) }
            }
        }
    }

    fun createWorkspace(vararg data: Pair<Language, String>): Workspace {
        val workspace = Workspace()
        for (i in 0 until data.size) {
            val uri = URI.create("file:///foo$i")
            workspace.create(uri, data[i].first, data[i].second)
        }
        return workspace
    }

    fun createRequest(workspace: Workspace, index: Int, line: Int, character: Int): CompletionRequest {
        val document = workspace.documents[index]
        val position = Position(line, character)
        val relatedDocuments = workspace.relatedDocuments(document.uri)
        return CompletionRequest(document.uri, relatedDocuments, position)
    }
}
