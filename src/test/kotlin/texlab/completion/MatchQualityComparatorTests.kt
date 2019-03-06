package texlab.completion

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import texlab.completion.latex.LatexKernelCommandProvider

class MatchQualityComparatorTests {
    @Test
    fun `it should prioritize items that begin with the query`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "\\usep")
                .completion("foo.tex", 0, 5)
        val comparator = MatchQualityComparator(request.document, request.params.position)
        val items = LatexKernelCommandProvider
                .get(request)
                .sortedWith(comparator)

        assertEquals("usepackage", items[0].label)
    }
}
