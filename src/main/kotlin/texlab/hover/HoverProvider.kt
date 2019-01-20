package texlab.hover

import org.eclipse.lsp4j.Hover

interface HoverProvider {
    fun getHover(request: HoverRequest): Hover?
}
