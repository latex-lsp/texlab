package texlab.references

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class BibtexEntryReferenceProviderTests {
    @Test
    fun `it should find citations in related documents`() = runBlocking<Unit> {
        val builder = WorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar = {baz}}")
                .document("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}")
                .document("baz.tex", "\\cite{foo}")

        val uri = builder.uri("bar.tex").toString()
        val range = Range(Position(1, 0), Position(1, 10))
        val location = Location(uri, range)
        builder.reference("foo.bib", 0, 9)
                .let { BibtexEntryReferenceProvider.get(it) }
                .also { assertEquals(1, it.size) }
                .also { assertEquals(location, it[0]) }
    }

    @Test
    fun `it should not process LaTeX documents`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.tex", "")
                .reference("foo.tex", 0, 0)
                .let { BibtexEntryReferenceProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
