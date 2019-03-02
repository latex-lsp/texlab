package texlab.provider

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class DistinctProviderTests {
    @Test
    fun `it should remove duplicates from the results of the given provider`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .request("foo.tex") {}

        val provider = object : FeatureProvider<Unit, Int> {
            override suspend fun get(request: FeatureRequest<Unit>): List<Int> {
                return List(2) { 0 }.plus(1)
            }
        }

        val result = DistinctProvider(provider) { it }.get(request)
        assertEquals(2, result.size)
    }
}
