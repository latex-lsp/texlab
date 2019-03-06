package texlab.provider

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
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

    @Test
    fun `it should return a result when a provider has a result`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .request("foo.tex") {}

        val firstProvider = NumberProvider(null)
        val secondProvider = NumberProvider(2)
        val provider = FeatureProvider.choice(firstProvider, secondProvider)
        val result = provider.get(request)
        assertEquals(result, secondProvider.number)
    }

    @Test
    fun `it should return nothing when no provider has a result`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .request("foo.tex") {}

        val firstProvider = NumberProvider(null)
        val secondProvider = NumberProvider(null)
        val provider = FeatureProvider.choice(firstProvider, secondProvider)
        val result = provider.get(request)
        assertNull(result)
    }
}
