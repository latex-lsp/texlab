package texlab.highlight

import org.eclipse.lsp4j.DocumentHighlight
import org.eclipse.lsp4j.DocumentHighlightKind
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class AggregateHighlightProviderTests {
    private fun createProvider(highlights: List<DocumentHighlight>?): HighlightProvider {
        return object : HighlightProvider {
            override fun getHighlights(request: HighlightRequest): List<DocumentHighlight>? {
                return highlights
            }
        }
    }

    @Test
    fun `it should return the first result`() {
        val provider1 = createProvider(null)
        val provider2 = createProvider(listOf(DocumentHighlight().apply { kind = DocumentHighlightKind.Read }))
        val provider3 = createProvider(listOf(DocumentHighlight().apply { kind = DocumentHighlightKind.Write }))
        val aggregateProvider = AggregateHighlightProvider(provider1, provider2, provider3)
        WorkspaceBuilder()
                .document("foo.tex", "")
                .highlight("foo.tex", 0, 0)
                .let { aggregateProvider.getHighlights(it) }
                .also { assertEquals(DocumentHighlightKind.Read, it!![0].kind) }
    }

    @Test
    fun `it should return null if no highlights were found`() {
        val provider1 = createProvider(null)
        val provider2 = createProvider(null)
        val aggregateProvider = AggregateHighlightProvider(provider1, provider2)
        WorkspaceBuilder()
                .document("foo.tex", "")
                .highlight("foo.tex", 0, 0)
                .let { aggregateProvider.getHighlights(it) }
                .also { assertNull(it) }
    }
}
