package texlab.link

import org.eclipse.lsp4j.DocumentLink

interface LinkProvider {
    fun getLinks(request: LinkRequest): List<DocumentLink>
}
