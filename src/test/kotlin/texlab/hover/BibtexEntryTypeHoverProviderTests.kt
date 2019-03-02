package texlab.hover

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class BibtexEntryTypeHoverProviderTests {
    @Test
    fun `it should return documentation when hovering over entry types`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "@article")
                .hover("foo.bib", 0, 2)
                .let { BibtexEntryTypeHoverProvider.get(it) }
                .also { assertTrue(it.isNotEmpty()) }
    }

    @Test
    fun `it should return nothing when not hovering over entry types`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar = {baz}}")
                .hover("foo.bib", 0, 10)
                .let { BibtexEntryTypeHoverProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }

    @Test
    fun `it should not process LaTeX documents`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.tex", "")
                .hover("foo.tex", 0, 0)
                .let { BibtexEntryTypeHoverProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
