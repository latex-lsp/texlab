package texlab.hover

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class BibtexEntryTypeHoverProviderTests {
    @Test
    fun `it should return documentation when hovering over entry types`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.bib", "@article")
                .hover("foo.bib", 0, 2)
                .let { BibtexEntryTypeHoverProvider.get(it) }
                .also { assertNotNull(it) }
    }

    @Test
    fun `it should return nothing when not hovering over entry types`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar = {baz}}")
                .hover("foo.bib", 0, 10)
                .let { BibtexEntryTypeHoverProvider.get(it) }
                .also { assertNull(it) }
    }

    @Test
    fun `it should not process LaTeX documents`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "")
                .hover("foo.tex", 0, 0)
                .let { BibtexEntryTypeHoverProvider.get(it) }
                .also { assertNull(it) }
    }
}
