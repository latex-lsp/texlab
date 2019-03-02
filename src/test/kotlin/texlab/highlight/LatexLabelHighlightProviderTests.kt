package texlab.highlight

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.DocumentHighlight
import org.eclipse.lsp4j.DocumentHighlightKind
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class LatexLabelHighlightProviderTests {
    @Test
    fun `it should highlight all references of a label`() = runBlocking<Unit> {
        val range1 = Range(Position(0, 7), Position(0, 10))
        val range2 = Range(Position(1, 5), Position(1, 8))
        val highlights = arrayOf(
                DocumentHighlight(range1, DocumentHighlightKind.Write),
                DocumentHighlight(range2, DocumentHighlightKind.Read))
        WorkspaceBuilder()
                .document("foo.tex", "\\label{foo}\n\\ref{foo}")
                .highlight("foo.tex", 0, 7)
                .let { LatexLabelHighlightProvider.get(it) }
                .also { assertArrayEquals(highlights, it.toTypedArray()) }
    }

    @Test
    fun `it should return nothing if no label is selected`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.tex", "")
                .highlight("foo.tex", 0, 0)
                .let { LatexLabelHighlightProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }

    @Test
    fun `it should not process BibTeX documents`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.bib", "")
                .highlight("foo.bib", 0, 0)
                .let { LatexLabelHighlightProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
