package texlab.rename

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class LatexCommandRenamerTests {
    @Test
    fun `it should rename commands in related documents`() = runBlocking {
        val builder = OldWorkspaceBuilder()
                .document("foo.tex", "\\include{bar.tex}\n\\baz")
                .document("bar.tex", "\\baz")

        val edit = builder
                .rename("foo.tex", 1, 2, "qux")
                .let { LatexCommandRenamer.get(it)!! }

        assertEquals(2, edit.changes.size)

        val document1 = builder.uri("foo.tex").toString()
        val change1 = edit.changes.getValue(document1)
        assertEquals(1, change1.size)
        assertEquals(Range(Position(1, 0), Position(1, 4)), change1[0].range)
        assertEquals("\\qux", change1[0].newText)

        val document2 = builder.uri("bar.tex").toString()
        val change2 = edit.changes.getValue(document2)
        assertEquals(1, change2.size)
        assertEquals(Range(Position(0, 0), Position(0, 4)), change2[0].range)
        assertEquals("\\qux", change2[0].newText)
    }

    @Test
    fun `it should not process BibTeX documents`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.bib", "\\foo \\bar")
                .rename("foo.bib", 0, 1, "baz")
                .let { LatexCommandRenamer.get(it) }
                .also { Assertions.assertNull(it) }
    }
}
