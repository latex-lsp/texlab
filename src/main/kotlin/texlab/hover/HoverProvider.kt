package texlab.hover

import org.eclipse.lsp4j.Hover

interface HoverProvider {
    suspend fun getHover(request: HoverRequest): Hover?
}
