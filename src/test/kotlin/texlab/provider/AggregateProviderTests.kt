package texlab.provider

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class AggregateProviderTests {
    private class NumberProvider(val number: Int) : FeatureProvider<Unit, Int> {
        override suspend fun get(request: FeatureRequest<Unit>): List<Int> {
            return listOf(number)
        }
    }

    @Test
    fun `it should concatenate the results of the given providers`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .request("foo.tex") {}

        val firstProvider = NumberProvider(1)
        val secondProvider = NumberProvider(2)
        val provider = AggregateProvider(firstProvider, secondProvider)
        val result = provider.get(request)
        assertEquals(2, result.size)
        assertEquals(firstProvider.number, result[0])
        assertEquals(secondProvider.number, result[1])
    }
}

