package texlab.completion

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import texlab.completion.latex.LatexKernelCommandProvider

class OrderByQualityProviderTests {
    @Test
    fun `it should prioritize items that begin with the query`() = runBlocking {
        val provider = OrderByQualityProvider(LatexKernelCommandProvider)
        val request = WorkspaceBuilder()
                .document("foo.tex", "\\usep")
                .completion("foo.tex", 0, 5)
        val items = provider.get(request)
        assertEquals("usepackage", items[0].label)
    }
}
