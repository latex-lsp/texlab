package texlab.completion.bibtex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class BibtexKernelCommandProviderTests {
    @Test
    fun `it should provide items when inside of a command`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar=\n\\}")
                .completion("foo.bib", 1, 1)
                .let { BibtexKernelCommandProvider.get(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when inside of text`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar=\n}")
                .completion("foo.bib", 1, 0)
                .let { BibtexKernelCommandProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items in LaTeX documents`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "\\")
                .completion("foo.tex", 0, 1)
                .let { BibtexKernelCommandProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
