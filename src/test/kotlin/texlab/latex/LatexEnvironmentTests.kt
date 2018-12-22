package texlab.latex

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class LatexEnvironmentTests {

    @Test
    fun `it should parse nested environments`() {
        val text = "\\begin{a}\\begin{b}\\end{c}\\end{d}"
        val tree = LatexParser.parse(text)
        val environment1 = LatexEnvironment(
                tree.children[1] as LatexCommandSyntax,
                tree.children[2] as LatexCommandSyntax)
        val environment2 = LatexEnvironment(
                tree.children[0] as LatexCommandSyntax,
                tree.children[3] as LatexCommandSyntax)
        val environments = LatexEnvironment.analyze(tree)
        assertEquals(2, environments.size)
        assertEquals(environment1, environments[0])
        assertEquals(environment2, environments[1])
    }

    @Test
    fun `it should parse environments with empty names`() {
        val text = "\\begin{}\\end{}"
        val tree = LatexParser.parse(text)
        val expected = LatexEnvironment(
                tree.children[0] as LatexCommandSyntax,
                tree.children[1] as LatexCommandSyntax)
        val environments = LatexEnvironment.analyze(tree)
        assertEquals(1, environments.size)
        assertEquals(expected, environments[0])
    }

    @Test
    fun `it should ignore unmatched delimiters`() {
        val text = "\\end{a}\\begin{b}"
        val tree = LatexParser.parse(text)
        val environments = LatexEnvironment.analyze(tree)
        assertEquals(0, environments.size)
    }

    @Test
    fun `it should ignore invalid delimiters`() {
        val text = "\\begin\\end"
        val tree = LatexParser.parse(text)
        val environments = LatexEnvironment.analyze(tree)
        assertEquals(0, environments.size)
    }
}
