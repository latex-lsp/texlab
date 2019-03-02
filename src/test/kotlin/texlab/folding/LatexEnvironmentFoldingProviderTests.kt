package texlab.folding

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.FoldingRange
import org.eclipse.lsp4j.FoldingRangeKind
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class LatexEnvironmentFoldingProviderTests {
    @Test
    fun `it should work with multiline environments`() = runBlocking<Unit> {
        val expected = arrayOf(FoldingRange().apply {
            startLine = 0
            startCharacter = 11
            endLine = 1
            endCharacter = 0
            kind = FoldingRangeKind.Region
        })

        WorkspaceBuilder()
                .document("foo.tex", "\\begin{foo}\n\\end{foo}")
                .folding("foo.tex")
                .let { LatexEnvironmentFoldingProvider.get(it) }
                .toTypedArray()
                .also { assertArrayEquals(expected, it) }
    }

    @Test
    fun `it should not process BibTeX documents`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "")
                .folding("foo.bib")
                .let { LatexEnvironmentFoldingProvider.get(it) }
                .also { assertEquals(0, it.size) }
    }
}
