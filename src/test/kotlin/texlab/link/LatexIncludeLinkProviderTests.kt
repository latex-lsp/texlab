package texlab.link

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import java.io.File

class LatexIncludeLinkProviderTests {
    @Test
    fun `it should provide links to related documents`() = runBlocking {
        val links = WorkspaceBuilder()
                .document("foo.tex", "\\input{bar.tex}")
                .document("bar.tex", "")
                .link("foo.tex")
                .let { LatexIncludeLinkProvider.get(it) }

        assertEquals(1, links.size)
        assertEquals(Range(Position(0, 7), Position(0, 14)), links[0].range)
        assertEquals(File("bar.tex").toURI().toString(), links[0].target)
    }

    @Test
    fun `it should not process BibTeX documents`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "")
                .link("foo.bib")
                .let { LatexIncludeLinkProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
