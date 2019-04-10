package texlab.rename

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.CsvSource
import texlab.OldWorkspaceBuilder

class BibtexEntryRenamerTests {
    @ParameterizedTest
    @CsvSource("foo.bib, 0, 9", "bar.tex, 1, 6")
    fun `it should be able to rename an entry`(document: String, line: Int, character: Int) = runBlocking {
        val builder = OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar = baz}")
                .document("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}")

        val edit = builder
                .rename(document, line, character, "qux")
                .let { BibtexEntryRenamer.get(it)!! }

        assertEquals(2, edit.changes.size)

        val document1 = builder.uri("foo.bib").toString()
        val change1 = edit.changes.getValue(document1)
        assertEquals(1, change1.size)
        assertEquals(Range(Position(0, 9), Position(0, 12)), change1[0].range)
        assertEquals("qux", change1[0].newText)

        val document2 = builder.uri("bar.tex").toString()
        val change2 = edit.changes.getValue(document2)
        assertEquals(1, change2.size)
        assertEquals(Range(Position(1, 6), Position(1, 9)), change2[0].range)
        assertEquals("qux", change2[0].newText)
    }

    @Test
    fun `it should not rename unrelated structures`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar = baz}")
                .rename("foo.bib", 0, 14, "qux")
                .let { BibtexEntryRenamer.get(it) }
                .also { assertNull(it) }
    }

    @Test
    fun `it should not process LaTeX documents`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "")
                .rename("foo.tex", 0, 0, "bar")
                .let { BibtexEntryRenamer.get(it) }
                .also { assertNull(it) }
    }
}
