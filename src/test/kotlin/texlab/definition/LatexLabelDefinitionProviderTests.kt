package texlab.definition

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class LatexLabelDefinitionProviderTests {
    @Test
    fun `it should find labels in related documents`() = runBlocking<Unit> {
        val builder = OldWorkspaceBuilder()
                .document("foo.tex", "\\label{foo}")
                .document("bar.tex", "\\label{foo}\n\\input{baz.tex}")
                .document("baz.tex", "\\ref{foo}")
        val uri = builder.uri("bar.tex").toString()
        val range = Range(Position(0, 7), Position(0, 10))
        val location = Location(uri, range)
        builder.definition("baz.tex", 0, 5)
                .let { LatexLabelDefinitionProvider.get(it).first() }
                .also { assertEquals(location, it) }
    }

    @Test
    fun `it should return nothing if no definition was found`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "")
                .definition("foo.tex", 0, 0)
                .let { LatexLabelDefinitionProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }

    @Test
    fun `it should not process BibTeX documents`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.bib", "")
                .definition("foo.bib", 0, 0)
                .let { LatexLabelDefinitionProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
