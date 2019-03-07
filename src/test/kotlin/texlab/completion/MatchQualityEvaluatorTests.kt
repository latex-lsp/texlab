package texlab.completion

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import texlab.completion.latex.LatexKernelCommandProvider

class MatchQualityEvaluatorTests {
    @Test
    fun `it should prioritize items that begin with the query`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "\\usep")
                .completion("foo.tex", 0, 5)

        val evaluator = MatchQualityEvaluator(request.document, request.params.position)
        val items = LatexKernelCommandProvider
                .get(request)
                .sortedByDescending { evaluator.evaluate(it) }

        assertEquals("usepackage", items[0].label)
    }
}
