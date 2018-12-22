package texlab

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.eclipse.lsp4j.TextDocumentContentChangeEvent
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import java.net.URI

class WorkspaceTests {
    private val document1 = URI.create("file:///foo/bar.tex")
    private val document2 = URI.create("file:///foo/baz/qux.tex")
    private val document3 = URI.create("file:///foo/baz.bib")

    private fun Workspace.verifyRelatedDocuments(expected: List<URI>) {
        val actual = relatedDocuments(expected[0]).map { it.uri }
        assertEquals(expected.size, actual.size)
        expected.zip(actual)
                .forEach { assertEquals(it.first, it.second) }
    }

    @Test
    fun `it should find related bibliographies`() {
        val workspace = Workspace()
        workspace.create(document1, Language.LATEX, "\\bibliography{baz.bib}")
        workspace.create(document2, Language.LATEX, "")
        workspace.create(document3, Language.BIBTEX, "")
        workspace.verifyRelatedDocuments(listOf(document3, document1))
    }

    @Test
    fun `it should append extensions when finding related documents`() {
        val workspace = Workspace()
        workspace.create(document1, Language.LATEX, "\\input{baz/qux}")
        workspace.create(document2, Language.LATEX, "")
        workspace.verifyRelatedDocuments(listOf(document1, document2))
    }

    @Test
    fun `it should not crash when including invalid paths`() {
        val workspace = Workspace()
        workspace.create(document1, Language.LATEX, "\\include{<foo>?|bar|:}")
        workspace.verifyRelatedDocuments(listOf(document1))
    }

    @Test
    fun `it should not crash when including empty paths`() {
        val workspace = Workspace()
        workspace.create(document1, Language.LATEX, "\\include{}")
        workspace.verifyRelatedDocuments(listOf(document1))
    }

    @Test
    fun `it should ignore paths that cannot be resolved`() {
        val workspace = Workspace()
        workspace.create(document1, Language.LATEX, "\\addbibresource{bar}")
        workspace.verifyRelatedDocuments(listOf(document1))
    }

    @Test
    fun `it should handle include cycles`() {
        val workspace = Workspace()
        workspace.create(document1, Language.LATEX, "\\include{baz/qux}");
        workspace.create(document2, Language.LATEX, "\\include{../bar.tex}");
        workspace.verifyRelatedDocuments(listOf(document1, document2))
    }

    @Test
    fun `it should handle incremental updates`() {
        val workspace = Workspace()
        workspace.create(document1, Language.LATEX, "foo\nbar")
        val range = Range(Position(0, 0), Position(0, 3))
        val change = TextDocumentContentChangeEvent(range, 3, "baz")
        workspace.update(document1, listOf(change), 1)
        assertEquals("baz\nbar", workspace.relatedDocuments(document1)[0].text)
    }
}
