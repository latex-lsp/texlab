package texlab.completion.latex

import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class TikzLibraryProviderTests {
    @Test
    fun `it should return all libraries inside of the import command`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\usetikzlibrary{}")
                .completion("foo.tex", 0, 16)
                .let { TikzLibraryProvider.complete(it) }
                .also { assertTrue(it.isNotEmpty()) }
    }

    @Test
    fun `it should return an empty list outside of the import command`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\usetikzlibrary{}")
                .completion("foo.tex", 0, 13)
                .let { TikzLibraryProvider.complete(it) }
                .also { assertTrue(it.isEmpty()) }
    }

    @Test
    fun `it should not process BibTeX documents`() {
        WorkspaceBuilder()
                .document("foo.bib", "")
                .completion("foo.bib", 0, 0)
                .let { TikzLibraryProvider.complete(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
