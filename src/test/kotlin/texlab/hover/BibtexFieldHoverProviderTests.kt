package texlab.hover

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class BibtexFieldHoverProviderTests {
    @Test
    fun `it should return documentation when hovering over entry types`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "@article{foo, author = }")
                .hover("foo.bib", 0, 15)
                .let { BibtexFieldHoverProvider.get(it) }
                .also { assertNotNull(it) }
    }

    @Test
    fun `it should return nothing when not hovering over entry types`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "@article{foo, author = {bar}}")
                .hover("foo.bib", 0, 5)
                .let { BibtexFieldHoverProvider.get(it) }
                .also { assertNull(it) }
    }

    @Test
    fun `it should not process LaTeX documents`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.tex", "")
                .hover("foo.tex", 0, 0)
                .let { BibtexFieldHoverProvider.get(it) }
                .also { assertNull(it) }
    }
}
