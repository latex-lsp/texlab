package texlab.syntax.latex

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class LatexSectionTests {
    @Test
    fun `it should find numbered sections`() {
        val text = "\\section{Foo Bar Baz}"
        val tree = LatexParser.parse(text)
        val expected = LatexSection(tree.children[0] as LatexCommandSyntax, "Foo Bar Baz", 1)
        val actual = LatexSection.find(tree)
        assertEquals(1, actual.size)
        assertEquals(expected, actual[0])
    }

    @Test
    fun `it should find unnumbered paragraphs`() {
        val text = "\\paragraph*{Foo}"
        val tree = LatexParser.parse(text)
        val expected = LatexSection(tree.children[0] as LatexCommandSyntax, "Foo", 4)
        val actual = LatexSection.find(tree)
        assertEquals(1, actual.size)
        assertEquals(expected, actual[0])
    }

    @Test
    fun `it should ignore invalid chapters`() {
        val text = "\\chapter*"
        val tree = LatexParser.parse(text)
        val actual = LatexSection.find(tree)
        assertEquals(0, actual.size)
    }
}
