package texlab.rename

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.CsvSource
import texlab.WorkspaceBuilder
import java.io.File

class LatexLabelRenamerTests {
    @ParameterizedTest
    @CsvSource("foo.tex, 0, 7", "bar.tex, 0, 5")
    fun `it should be able to rename a label`(document: String, line: Int, character: Int) {
        val edit = WorkspaceBuilder()
                .document("foo.tex", "\\label{foo}\n\\include{bar}")
                .document("bar.tex", "\\ref{foo}")
                .rename(document, line, character, "bar")
                .let { LatexLabelRenamer.rename(it) }!!

        Assertions.assertEquals(2, edit.changes.size)

        val document1 = File("foo.tex").toURI().toString()
        val change1 = edit.changes.getValue(document1)
        assertEquals(1, change1.size)
        assertEquals(Range(Position(0, 7), Position(0, 10)), change1[0].range)
        assertEquals("bar", change1[0].newText)

        val document2 = File("bar.tex").toURI().toString()
        val change2 = edit.changes.getValue(document2)
        assertEquals(1, change2.size)
        assertEquals(Range(Position(0, 5), Position(0, 8)), change2[0].range)
        assertEquals("bar", change2[0].newText)
    }

    @Test
    fun `it should not rename unrelated structures`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\foo{bar}")
                .rename("foo.tex", 0, 5, "baz")
                .let { BibtexEntryRenamer.rename(it) }
                .also { assertNull(it) }
    }
}
