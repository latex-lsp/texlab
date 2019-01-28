package texlab.definition

import org.eclipse.lsp4j.Location
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class AggregateDefinitionProviderTests {
    private fun createProvider(location: Location?): DefinitionProvider {
        return object : DefinitionProvider {
            override fun find(request: DefinitionRequest): Location? {
                return location
            }
        }
    }

    @Test
    fun `it should return the first result`() {
        val provider1 = createProvider(null)
        val provider2 = createProvider(Location().apply { uri = "foo.tex" })
        val provider3 = createProvider(Location().apply { uri = "bar.tex" })
        val aggregateProvider = AggregateDefinitionProvider(provider1, provider2, provider3)
        WorkspaceBuilder()
                .document("foo.tex", "")
                .definition("foo.tex", 0, 0)
                .let { aggregateProvider.find(it) }
                .also { assertEquals("foo.tex", it!!.uri) }
    }

    @Test
    fun `it should return null if no definition was found`() {
        val provider1 = createProvider(null)
        val provider2 = createProvider(null)
        val aggregateProvider = AggregateDefinitionProvider(provider1, provider2)
        WorkspaceBuilder()
                .document("foo.tex", "")
                .definition("foo.tex", 0, 0)
                .let { aggregateProvider.find(it) }
                .also { assertNull(it) }
    }
}
