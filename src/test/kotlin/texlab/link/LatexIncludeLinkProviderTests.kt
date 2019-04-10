package texlab.link

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class LatexIncludeLinkProviderTests {
    @Test
    fun `it should provide links to related documents`() = runBlocking {
        val builder = OldWorkspaceBuilder()
                .document("foo.tex", "\\input{bar.tex}")
                .document("bar.tex", "")

        val links = builder
                .link("foo.tex")
                .let { LatexIncludeLinkProvider.get(it) }

        assertEquals(1, links.size)
        assertEquals(Range(Position(0, 7), Position(0, 14)), links[0].range)
        assertEquals(builder.uri("bar.tex").toString(), links[0].target)
    }

    @Test
    fun `it should not process BibTeX documents`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.bib", "")
                .link("foo.bib")
                .let { LatexIncludeLinkProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
