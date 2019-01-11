package texlab.folding

import org.eclipse.lsp4j.FoldingRange
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class AggregateFoldingProviderTests {
    private fun createProvider(vararg foldings: FoldingRange): FoldingProvider {
        return object : FoldingProvider {
            override fun fold(request: FoldingRequest): List<FoldingRange> {
                return foldings.toList()
            }
        }
    }

    @Test
    fun `it should merge foldings from all providers`() {
        val folding1 = FoldingRange().apply { startLine = 1 }
        val folding2 = FoldingRange().apply { startLine = 2 }
        val folding3 = FoldingRange().apply { startLine = 3 }
        val provider1 = createProvider(folding1, folding2)
        val provider2 = createProvider(folding3)
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .folding("foo.tex")

        val aggregateProvider = AggregateFoldingProvider(provider1, provider2)
        val foldings = aggregateProvider.fold(request)
        assertArrayEquals(arrayOf(folding1, folding2, folding3), foldings.toTypedArray())
    }
}
