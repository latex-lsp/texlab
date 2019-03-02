package texlab.references

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import java.io.File

class LatexLabelReferenceProviderTests {
    @Test
    fun `it should find labels in related documents`() = runBlocking<Unit> {
        val uri = File("bar.tex").toURI().toString()
        val range = Range(Position(1, 0), Position(1, 9))
        val location = Location(uri, range)
        WorkspaceBuilder()
                .document("foo.tex", "\\label{foo}")
                .document("bar.tex", "\\input{foo.tex}\n\\ref{foo}")
                .document("baz.tex", "\\ref{foo}")
                .reference("foo.tex", 0, 8)
                .let { LatexLabelReferenceProvider.get(it) }
                .also { assertEquals(1, it.size) }
                .also { assertEquals(location, it[0]) }
    }

    @Test
    fun `it should not process BibTeX documents`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "")
                .reference("foo.bib", 0, 0)
                .let { LatexLabelReferenceProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
