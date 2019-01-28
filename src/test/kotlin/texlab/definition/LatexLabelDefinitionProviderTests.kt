package texlab.definition

import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import java.io.File

class LatexLabelDefinitionProviderTests {
    @Test
    fun `it should find labels in related documents`() {
        val uri = File("bar.tex").toURI().toString()
        val range = Range(Position(0, 7), Position(0, 10))
        val location = Location(uri, range)
        WorkspaceBuilder()
                .document("foo.tex", "\\label{foo}")
                .document("bar.tex", "\\label{foo}\n\\input{baz.tex}")
                .document("baz.tex", "\\ref{foo}")
                .definition("baz.tex", 0, 5)
                .let { LatexLabelDefinitionProvider.find(it) }
                .also { assertEquals(location, it) }
    }

    @Test
    fun `it should return null if no definition was found`() {
        WorkspaceBuilder()
                .document("foo.tex", "")
                .definition("foo.tex", 0, 0)
                .let { LatexLabelDefinitionProvider.find(it) }
                .also { assertNull(it) }
    }

    @Test
    fun `it should not process BibTeX documents`() {
        WorkspaceBuilder()
                .document("foo.bib", "")
                .definition("foo.bib", 0, 0)
                .let { LatexLabelDefinitionProvider.find(it) }
                .also { assertNull(it) }
    }
}
