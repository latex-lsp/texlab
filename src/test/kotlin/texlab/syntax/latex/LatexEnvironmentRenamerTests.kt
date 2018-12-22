package texlab.syntax.latex

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.eclipse.lsp4j.TextEdit
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test

class LatexEnvironmentRenamerTests {

    @Test
    fun `it should rename environments with different names`() {
        val text = "\\begin{foo}\n\\end{bar}"
        val tree = LatexSyntaxTree(text)
        val edit1 = TextEdit(Range(Position(0, 7), Position(0, 10)), "baz")
        val edit2 = TextEdit(Range(Position(1, 5), Position(1, 8)), "baz")
        val actual = LatexEnvironmentRenamer.rename(tree, Position(0, 7), "baz")!!
        assertEquals(2, actual.size)
        assertEquals(edit1, actual[0])
        assertEquals(edit2, actual[1])
    }

    @Test
    fun `it should rename environments with empty names`() {
        val text = "\\begin{foo}\n\\end{}"
        val tree = LatexSyntaxTree(text)
        val edit1 = TextEdit(Range(Position(0, 7), Position(0, 10)), "baz")
        val edit2 = TextEdit(Range(Position(1, 5), Position(1, 5)), "baz")
        val actual = LatexEnvironmentRenamer.rename(tree, Position(0, 7), "baz")!!
        assertEquals(2, actual.size)
        assertEquals(edit1, actual[0])
        assertEquals(edit2, actual[1])
    }

    @Test
    fun `it should ignore unrelated tokens`() {
        val text = "\\begin{foo}\n\\end{bar}"
        val tree = LatexSyntaxTree(text)
        val actual = LatexEnvironmentRenamer.rename(tree, Position(0, 1), "baz")
        assertNull(actual)
    }
}
