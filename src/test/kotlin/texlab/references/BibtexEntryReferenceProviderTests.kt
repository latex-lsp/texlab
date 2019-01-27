package texlab.references

import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import java.io.File

class BibtexEntryReferenceProviderTests {
    @Test
    fun `it should find citations in related documents`() {
        val uri = File("bar.tex").toURI().toString()
        val range = Range(Position(1, 0), Position(1, 10))
        val location = Location(uri, range)
        WorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar = {baz}}")
                .document("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}")
                .document("baz.tex", "\\cite{foo}")
                .reference("foo.bib", 0, 9)
                .let { BibtexEntryReferenceProvider.getReferences(it) }
                .also { assertEquals(1, it?.size) }
                .also { assertEquals(location, it!![0]) }
    }

    @Test
    fun `it should not process LaTeX documents`() {
        WorkspaceBuilder()
                .document("foo.tex", "")
                .reference("foo.tex", 0, 0)
                .let { BibtexEntryReferenceProvider.getReferences(it) }
                .also { assertNull(it) }
    }
}
