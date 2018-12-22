package texlab.latex

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class LatexIncludeTests {

    @Test
    fun `it should find includes`() {
        val text = "\\include{foo}\n\\input{bar/qux}"
        val root = LatexParser.parse(text)
        val include1 = LatexInclude(root.children[0] as LatexCommandSyntax, "foo")
        val include2 = LatexInclude(root.children[1] as LatexCommandSyntax, "bar/qux")
        val includes = LatexInclude.analyze(root)
        assertEquals(2, includes.size)
        assertEquals(include1, includes[0])
        assertEquals(include2, includes[1])
    }

    @Test
    fun `it should find bibliographies`() {
        val text = "\\addbibresource{foo}\n\\bibliography{bar}"
        val root = LatexParser.parse(text)
        val include1 = LatexInclude(root.children[0] as LatexCommandSyntax, "foo")
        val include2 = LatexInclude(root.children[1] as LatexCommandSyntax, "bar")
        val includes = LatexInclude.analyze(root)
        assertEquals(2, includes.size)
        assertEquals(include1, includes[0])
        assertEquals(include2, includes[1])
    }

    @Test
    fun `it should handle paths with spaces`() {
        val text = "\\include{foo bar.tex}"
        val root = LatexParser.parse(text)
        val expected = LatexInclude(root.children[0] as LatexCommandSyntax, "foo bar.tex")
        val includes = LatexInclude.analyze(root)
        assertEquals(1, includes.size)
        assertEquals(expected, includes[0])
    }

    @Test
    fun `it should ignore invalid commands`() {
        val text = "\\include \\input{}"
        val root = LatexParser.parse(text)
        assertEquals(0, LatexInclude.analyze(root).size)
    }
}
