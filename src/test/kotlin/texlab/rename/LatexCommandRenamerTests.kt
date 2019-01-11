package texlab.rename

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import java.io.File

class LatexCommandRenamerTests {
    @Test
    fun `it should rename commands in related documents`() {
        val edit = WorkspaceBuilder()
                .document("foo.tex", "\\include{bar.tex}\n\\baz")
                .document("bar.tex", "\\baz")
                .rename("foo.tex", 1, 2, "qux")
                .let { LatexCommandRenamer.rename(it) }!!

        assertEquals(2, edit.changes.size)

        val document1 = File("foo.tex").toURI().toString()
        val change1 = edit.changes.getValue(document1)
        assertEquals(1, change1.size)
        assertEquals(Range(Position(1, 0), Position(1, 4)), change1[0].range)
        assertEquals("\\qux", change1[0].newText)

        val document2 = File("bar.tex").toURI().toString()
        val change2 = edit.changes.getValue(document2)
        assertEquals(1, change2.size)
        assertEquals(Range(Position(0, 0), Position(0, 4)), change2[0].range)
        assertEquals("\\qux", change2[0].newText)
    }

    @Test
    fun `it should not process BibTeX documents`() {
        WorkspaceBuilder()
                .document("foo.bib", "\\foo \\bar")
                .rename("foo.bib", 0, 1, "baz")
                .let { LatexCommandRenamer.rename(it) }
                .also { Assertions.assertNull(it) }
    }
}
