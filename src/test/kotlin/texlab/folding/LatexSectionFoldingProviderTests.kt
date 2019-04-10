package texlab.folding

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.FoldingRange
import org.eclipse.lsp4j.FoldingRangeKind
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class LatexSectionFoldingProviderTests {
    @Test
    fun `it should work with multiple levels of nesting`() = runBlocking<Unit> {
        val text = "\\section{Foo}\nfoo\n\\subsection{Bar}\nbar\n\\section{Baz}\nbaz\n\\section{Qux}"

        val folding1 = FoldingRange().apply {
            startLine = 0
            startCharacter = 13
            endLine = 3
            endCharacter = 0
            kind = FoldingRangeKind.Region
        }
        val folding2 = FoldingRange().apply {
            startLine = 2
            startCharacter = 16
            endLine = 3
            endCharacter = 0
            kind = FoldingRangeKind.Region
        }
        val folding3 = FoldingRange().apply {
            startLine = 4
            startCharacter = 13
            endLine = 5
            endCharacter = 0
            kind = FoldingRangeKind.Region
        }
        val expected = arrayOf(folding1, folding2, folding3)

        OldWorkspaceBuilder()
                .document("foo.tex", text)
                .folding("foo.tex")
                .let { LatexSectionFoldingProvider.get(it) }
                .sortedBy { it.startLine }
                .toTypedArray()
                .also { assertArrayEquals(expected, it) }
    }

    @Test
    fun `it should not process BibTeX documents`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.bib", "")
                .folding("foo.bib")
                .let { LatexSectionFoldingProvider.get(it) }
                .also { Assertions.assertEquals(0, it.size) }
    }
}
