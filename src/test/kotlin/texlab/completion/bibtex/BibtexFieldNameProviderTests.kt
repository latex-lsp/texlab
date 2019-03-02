package texlab.completion.bibtex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class BibtexFieldNameProviderTests {
    @Test
    fun `it should provide items when inside of the first field`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "@article{foo,\nbar}")
                .completion("foo.bib", 1, 1)
                .let { BibtexFieldNameProvider.get(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should provide items when inside of the second field`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar = {baz}, qux}")
                .completion("foo.bib", 0, 27)
                .let { BibtexFieldNameProvider.get(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should provide items when inside of an entry`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "@article{foo, \n}")
                .completion("foo.bib", 1, 0)
                .let { BibtexFieldNameProvider.get(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when inside content`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "@article{foo,\nbar = {baz}}")
                .completion("foo.bib", 1, 7)
                .let { BibtexFieldNameProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when inside entry types`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "@article{foo,}")
                .completion("foo.bib", 0, 3)
                .let { BibtexFieldNameProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
