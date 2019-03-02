package texlab.provider

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class LimitedProviderTests {
    @Test
    fun `it should limit the results of the given provider`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .request("foo.tex") {}

        val provider = object : FeatureProvider<Unit, Int> {
            override suspend fun get(request: FeatureRequest<Unit>): List<Int> {
                return List(100) { it }
            }
        }

        val limit = 50
        val result = LimitedProvider(provider, limit).get(request)
        Assertions.assertEquals(limit, result.size)
    }
}

