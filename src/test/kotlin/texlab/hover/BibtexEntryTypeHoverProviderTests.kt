package texlab.hover

import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class BibtexEntryTypeHoverProviderTests {
    @Test
    fun `it should return documentation when hovering over entry types`() {
        WorkspaceBuilder()
                .document("foo.bib", "@article")
                .hover("foo.bib", 0, 2)
                .let { BibtexEntryTypeHoverProvider.getHover(it) }
                .also { assertNotNull(it) }
    }

    @Test
    fun `it should return null when not hovering over entry types`() {
        WorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar = {baz}}")
                .hover("foo.bib", 0, 10)
                .let { BibtexEntryTypeHoverProvider.getHover(it) }
                .also { assertNull(it) }
    }

    @Test
    fun `it should not process LaTeX documents`() {
        WorkspaceBuilder()
                .document("foo.tex", "")
                .hover("foo.tex", 0, 0)
                .let { BibtexEntryTypeHoverProvider.getHover(it) }
                .also { assertNull(it) }
    }
}
