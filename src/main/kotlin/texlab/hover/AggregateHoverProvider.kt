package texlab.hover

import org.eclipse.lsp4j.Hover

class AggregateHoverProvider(private vararg val providers: HoverProvider) : HoverProvider {
    override fun getHover(request: HoverRequest): Hover? {
        for (provider in providers) {
            val hover = provider.getHover(request)
            if (hover != null) {
                return hover
            }
        }
        return null
    }
}
