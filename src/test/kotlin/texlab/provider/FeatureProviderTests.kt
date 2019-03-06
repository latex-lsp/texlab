package texlab.provider

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class FeatureProviderTests {
    class NumberProvider(val number: Int?) : FeatureProvider<Unit, Int?> {
        override suspend fun get(request: FeatureRequest<Unit>): Int? {
            return number
        }
    }

    class NumberListProvider(val number: Int) : FeatureProvider<Unit, List<Int>> {
        override suspend fun get(request: FeatureRequest<Unit>): List<Int> {
            return listOf(number)
        }
    }

    @Test
    fun `it should create a new provider that maps the result`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .request("foo.tex") {}

        val provider = NumberProvider(1)
        val firstResult = provider.get(request)

        val transform = { x: Int? -> x?.plus(1) }
        val result = provider
                .map { transform(it) }
                .get(request)

        assertNotNull(result)
        assertEquals(transform(firstResult), result)
    }

    @Test
    fun `it should concatenate the results of the given providers`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .request("foo.tex") {}

        val firstProvider = NumberListProvider(1)
        val secondProvider = NumberListProvider(2)
        val provider = FeatureProvider.concat(firstProvider, secondProvider)
        val result = provider.get(request)
        assertEquals(2, result.size)
        assertEquals(firstProvider.number, result[0])
        assertEquals(secondProvider.number, result[1])
    }
}
