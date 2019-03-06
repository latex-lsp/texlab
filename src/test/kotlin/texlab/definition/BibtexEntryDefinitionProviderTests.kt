package texlab.definition

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class BibtexEntryDefinitionProviderTests {
    @Test
    fun `it should find entries in related documents`() = runBlocking<Unit> {
        val builder = WorkspaceBuilder()
                .document("foo.tex", "\\addbibresource{baz.bib}\n\\cite{foo}")
                .document("bar.bib", "@article{foo, bar = {baz}}")
                .document("baz.bib", "@article{foo, bar = {baz}}")

        val uri = builder.uri("baz.bib").toString()
        val range = Range(Position(0, 9), Position(0, 12))
        val location = Location(uri, range)
        builder.definition("foo.tex", 1, 6)
                .let { BibtexEntryDefinitionProvider.get(it).first() }
                .also { assertEquals(location, it) }
    }

    @Test
    fun `it should return nothing if no definition was found`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.tex", "")
                .definition("foo.tex", 0, 0)
                .let { BibtexEntryDefinitionProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }

    @Test
    fun `it should not process BibTeX documents`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "")
                .definition("foo.bib", 0, 0)
                .let { BibtexEntryDefinitionProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
