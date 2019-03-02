package texlab.completion.bibtex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class BibtexEntryTypeProviderTests {
    @Test
    fun `it should provide items when near the at sign`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "@")
                .completion("foo.bib", 0, 1)
                .let { BibtexEntryTypeProvider.get(it) }
                .also { assertTrue(it.isNotEmpty()) }
    }

    @Test
    fun `it should not provide items when inside of content`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar=\n@}")
                .completion("foo.bib", 1, 1)
                .let { BibtexEntryTypeProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items in LaTeX documents`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.tex", "@")
                .completion("foo.tex", 0, 1)
                .let { BibtexEntryTypeProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
