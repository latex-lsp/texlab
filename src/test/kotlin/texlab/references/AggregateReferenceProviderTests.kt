package texlab.references

import org.eclipse.lsp4j.Location
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class AggregateReferenceProviderTests {
    private fun createProvider(locations: List<Location>?): ReferenceProvider {
        return object : ReferenceProvider {
            override fun getReferences(request: ReferenceRequest): List<Location>? {
                return locations?.toList()
            }
        }
    }

    @Test
    fun `it should return the first result`() {
        val provider1 = createProvider(null)
        val provider2 = createProvider(listOf(Location().apply { uri = "foo.tex" }))
        val provider3 = createProvider(listOf(Location().apply { uri = "bar.tex" }))
        val aggregateProvider = AggregateReferenceProvider(provider1, provider2, provider3)
        WorkspaceBuilder()
                .document("foo.tex", "")
                .reference("foo.tex", 0, 0)
                .let { aggregateProvider.getReferences(it) }
                .also { assertEquals("foo.tex", it!![0].uri) }
    }

    @Test
    fun `it should return null if no references were found`() {
        val provider1 = createProvider(null)
        val provider2 = createProvider(null)
        val aggregateProvider = AggregateReferenceProvider(provider1, provider2)
        WorkspaceBuilder()
                .document("foo.tex", "")
                .reference("foo.tex", 0, 0)
                .let { aggregateProvider.getReferences(it) }
                .also { assertNull(it) }
    }
}
