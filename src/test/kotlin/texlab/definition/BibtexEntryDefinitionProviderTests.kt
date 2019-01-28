package texlab.definition

import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import java.io.File

class BibtexEntryDefinitionProviderTests {
    @Test
    fun `it should find entries in related documents`() {
        val uri = File("baz.bib").toURI().toString()
        val range = Range(Position(0, 9), Position(0, 12))
        val location = Location(uri, range)
        WorkspaceBuilder()
                .document("foo.tex", "\\addbibresource{baz.bib}\n\\cite{foo}")
                .document("bar.bib", "@article{foo, bar = {baz}}")
                .document("baz.bib", "@article{foo, bar = {baz}}")
                .definition("foo.tex", 1, 6)
                .let { BibtexEntryDefinitionProvider.find(it) }
                .also { assertEquals(location, it) }
    }

    @Test
    fun `it should return null if no definition was found`() {
        WorkspaceBuilder()
                .document("foo.tex", "")
                .definition("foo.tex", 0, 0)
                .let { BibtexEntryDefinitionProvider.find(it) }
                .also { assertNull(it) }
    }

    @Test
    fun `it should not process BibTeX documents`() {
        WorkspaceBuilder()
                .document("foo.bib", "")
                .definition("foo.bib", 0, 0)
                .let { BibtexEntryDefinitionProvider.find(it) }
                .also { assertNull(it) }
    }
}
