package texlab.link

import org.eclipse.lsp4j.DocumentLink

class AggregateLinkProvider(private vararg val providers: LinkProvider) : LinkProvider {
    override fun getLinks(request: LinkRequest): List<DocumentLink> {
        return providers.flatMap { it.getLinks(request) }
    }
}

