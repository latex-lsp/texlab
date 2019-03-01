package texlab.hover

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Hover
import org.eclipse.lsp4j.jsonrpc.messages.Either
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class AggregateHoverProviderTests {
    private fun createProvider(hover: Hover?): HoverProvider {
        return object : HoverProvider {
            override suspend fun getHover(request: HoverRequest): Hover? {
                return hover
            }
        }
    }

    @Test
    fun `it should return the first result`() = runBlocking<Unit> {
        val provider1 = createProvider(null)
        val provider2 = createProvider(Hover(listOf(Either.forLeft("foo"))))
        val provider3 = createProvider(Hover(listOf(Either.forLeft("bar"))))
        val aggregateProvider = AggregateHoverProvider(provider1, provider2, provider3)
        WorkspaceBuilder()
                .document("foo.tex", "")
                .hover("foo.tex", 0, 0)
                .let { aggregateProvider.getHover(it) }
                .also { assertEquals("foo", it!!.contents.left[0].left) }
    }

    @Test
    fun `it should return null if no hover information was found`() = runBlocking<Unit> {
        val provider1 = createProvider(null)
        val provider2 = createProvider(null)
        val aggregateProvider = AggregateHoverProvider(provider1, provider2)
        WorkspaceBuilder()
                .document("foo.tex", "")
                .hover("foo.tex", 0, 0)
                .let { aggregateProvider.getHover(it) }
                .also { assertNull(it) }
    }
}
