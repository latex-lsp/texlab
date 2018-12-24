package texlab.completion

import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.Language

class AggregateProviderTests {

    private fun verifyItems(provider1: CompletionProvider, provider2: CompletionProvider, vararg expected: String) {
        val aggregate = AggregateProvider(provider1, provider2)
        val workspace = CompletionTestsHelper.createWorkspace(Language.LATEX to "foo")
        val request = CompletionTestsHelper.createRequest(workspace, 0, 0, 0)
        val actual = aggregate.getItems(request).map { it.label }.toTypedArray()
        assertArrayEquals(expected, actual)
    }

    @Test
    fun `it should merge the results of all providers`() {
        val provider1 = CompletionTestsHelper.createProvider("Item1")
        val provider2 = CompletionTestsHelper.createProvider("Item2", "Item3")
        verifyItems(provider1, provider2, "Item1", "Item2", "Item3")
    }

    @Test
    fun `it should remove duplicates`() {
        val provider1 = CompletionTestsHelper.createProvider("Item1", "Item2")
        val provider2 = CompletionTestsHelper.createProvider("Item2", "Item3")
        verifyItems(provider1, provider2, "Item1", "Item2", "Item3")
    }
}
