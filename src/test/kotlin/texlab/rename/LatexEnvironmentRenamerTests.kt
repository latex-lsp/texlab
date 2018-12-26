package texlab.rename

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class LatexEnvironmentRenamerTests {
    @Test
    fun `it should rename unmatched environments`() {
        val edit = WorkspaceBuilder()
                .document("foo.tex", "\\begin{foo}\n\\end{bar}")
                .rename("foo.tex", 0, 8, "baz")
                .let { LatexEnvironmentRenamer.rename(it) }!!

        assertEquals(1, edit.changes.keys.size)
        val changes = edit.changes.getValue(edit.changes.keys.first())
        assertEquals(2, changes.size)
        assertEquals(Range(Position(0, 7), Position(0, 10)), changes[0].range)
        assertEquals("baz", changes[0].newText)
        assertEquals(Range(Position(1, 5), Position(1, 8)), changes[1].range)
        assertEquals("baz", changes[1].newText)
    }

    @Test
    fun `it should not rename unrelated environments`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\begin{foo}\n\\end{bar}")
                .rename("foo.tex", 0, 5, "baz")
                .let { LatexEnvironmentRenamer.rename(it) }
                .also { assertNull(it) }
    }

    @Test
    fun `it should not process BibTeX documents`() {
        WorkspaceBuilder()
                .document("foo.bib", "\\begin{foo}\n\\end{bar}")
                .rename("foo.bib", 0, 8, "baz")
                .let { LatexEnvironmentRenamer.rename(it) }
                .also { assertNull(it) }
    }
}
