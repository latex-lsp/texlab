package texlab.rename

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.CsvSource
import texlab.WorkspaceBuilder

class LatexLabelRenamerTests {
    @ParameterizedTest
    @CsvSource("foo.tex, 0, 7", "bar.tex, 0, 5")
    fun `it should be able to rename a label`(document: String, line: Int, character: Int) = runBlocking {
        val builder = WorkspaceBuilder()
                .document("foo.tex", "\\label{foo}\n\\include{bar}")
                .document("bar.tex", "\\ref{foo}")
        val edit = builder
                .rename(document, line, character, "bar")
                .let { LatexLabelRenamer.get(it).first() }

        Assertions.assertEquals(2, edit.changes.size)

        val document1 = builder.uri("foo.tex").toString()
        val change1 = edit.changes.getValue(document1)
        assertEquals(1, change1.size)
        assertEquals(Range(Position(0, 7), Position(0, 10)), change1[0].range)
        assertEquals("bar", change1[0].newText)

        val document2 = builder.uri("bar.tex").toString()
        val change2 = edit.changes.getValue(document2)
        assertEquals(1, change2.size)
        assertEquals(Range(Position(0, 5), Position(0, 8)), change2[0].range)
        assertEquals("bar", change2[0].newText)
    }

    @Test
    fun `it should not rename unrelated structures`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.tex", "\\foo{bar}")
                .rename("foo.tex", 0, 5, "baz")
                .let { LatexLabelRenamer.get(it).firstOrNull() }
                .also { assertNull(it) }
    }

    @Test
    fun `it should process BibTeX documents`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "")
                .rename("foo.bib", 0, 0, "bar")
                .let { LatexLabelRenamer.get(it).firstOrNull() }
                .also { assertNull(it) }
    }
}
