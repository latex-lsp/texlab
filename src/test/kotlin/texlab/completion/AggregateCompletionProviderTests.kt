package texlab.completion

import org.eclipse.lsp4j.CompletionItem
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class AggregateCompletionProviderTests {
    private fun createProvider(vararg items: String): CompletionProvider {
        return object : CompletionProvider {
            override fun complete(request: CompletionRequest): List<CompletionItem> {
                return items.map { CompletionItem(it) }
            }
        }
    }

    @Test
    fun `it should remove duplicates`() {
        val provider1 = createProvider("Item1", "Item2")
        val provider2 = createProvider("Item2", "Item3")
        val aggregate = AggregateCompletionProvider(provider1, provider2)

        val expected = arrayOf("Item1", "Item2", "Item3")
        val actual = WorkspaceBuilder()
                .document("foo.tex", "")
                .completion("foo.tex", 0, 0)
                .let { aggregate.complete(it) }
                .map { it.label }
                .toTypedArray()
        assertArrayEquals(expected, actual)
    }
}
