package texlab.provider

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class FeatureProviderTests {
    class NumberProvider(val number: Int) : FeatureProvider<Unit, Int> {
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

        val transform = { items: List<Int> -> items.map { it + 1 } }
        val result = provider
                .map { transform(it) }
                .get(request)

        Assertions.assertEquals(1, result.size)
        Assertions.assertEquals(transform(firstResult)[0], result[0])
    }

    @Test
    fun `it should concatenate the results of the given providers`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .request("foo.tex") {}

        val firstProvider = NumberProvider(1)
        val secondProvider = NumberProvider(2)
        val provider = FeatureProvider.concat(firstProvider, secondProvider)
        val result = provider.get(request)
        Assertions.assertEquals(2, result.size)
        Assertions.assertEquals(firstProvider.number, result[0])
        Assertions.assertEquals(secondProvider.number, result[1])
    }
}
