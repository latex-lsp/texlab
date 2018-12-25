package texlab.syntax.latex

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class LatexParserTests {
    private fun range(startLine: Int, startCharacter: Int, endLine: Int, endCharacter: Int): Range {
        val start = Position(startLine, startCharacter)
        val end = Position(endLine, endCharacter)
        return Range(start, end)
    }

    @Test
    fun `it should parse commands without arguments and options`() {
        val text = "\\foo"
        val tree =
                LatexDocumentSyntax(
                        range(0, 0, 0, 4),
                        listOf(
                                LatexCommandSyntax(
                                        range(0, 0, 0, 4),
                                        name = LatexToken(0, 0, "\\foo", LatexTokenKind.COMMAND),
                                        options = null,
                                        args = listOf())))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should parse commands with options`() {
        val text = "\\foo[bar]"
        val opts =
                LatexGroupSyntax(
                        range(0, 4, 0, 9),
                        left = LatexToken(0, 4, "[", LatexTokenKind.BEGIN_OPTIONS),
                        right = LatexToken(0, 8, "]", LatexTokenKind.END_OPTIONS),
                        children = listOf(LatexTextSyntax(
                                range(0, 5, 0, 8),
                                listOf(LatexToken(0, 5, "bar", LatexTokenKind.WORD)))))
        val tree =
                LatexDocumentSyntax(
                        range(0, 0, 0, 9),
                        listOf(
                                LatexCommandSyntax(
                                        range(0, 0, 0, 9),
                                        name = LatexToken(0, 0, "\\foo", LatexTokenKind.COMMAND),
                                        options = opts,
                                        args = listOf())))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should parse commands with empty arguments`() {
        val text = "\\foo{}"
        val args =
                LatexGroupSyntax(
                        range(0, 4, 0, 6),
                        left = LatexToken(0, 4, "{", LatexTokenKind.BEGIN_GROUP),
                        right = LatexToken(0, 5, "}", LatexTokenKind.END_GROUP),
                        children = listOf())
        val tree =
                LatexDocumentSyntax(
                        range(0, 0, 0, 6),
                        listOf(
                                LatexCommandSyntax(
                                        range = range(0, 0, 0, 6),
                                        name = LatexToken(0, 0, "\\foo", LatexTokenKind.COMMAND),
                                        options = null,
                                        args = listOf(args))))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should parse commands with text arguments`() {
        val text = "\\begin{foo}"
        val args = listOf(
                LatexGroupSyntax(
                        range(0, 6, 0, 11),
                        left = LatexToken(0, 6, "{", LatexTokenKind.BEGIN_GROUP),
                        right = LatexToken(0, 10, "}", LatexTokenKind.END_GROUP),
                        children = listOf(
                                LatexTextSyntax(
                                        range(0, 7, 0, 10),
                                        listOf(LatexToken(0, 7, "foo", LatexTokenKind.WORD))))))
        val tree =
                LatexDocumentSyntax(
                        range(0, 0, 0, 11),
                        listOf(
                                LatexCommandSyntax(
                                        range(0, 0, 0, 11),
                                        name = LatexToken(0, 0, "\\begin", LatexTokenKind.COMMAND),
                                        options = null,
                                        args = args)))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should parse text`() {
        val text = "foo bar"
        val tree =
                LatexDocumentSyntax(
                        range(0, 0, 0, 7),
                        listOf(
                                LatexTextSyntax(
                                        range(0, 0, 0, 7),
                                        listOf(
                                                LatexToken(0, 0, "foo", LatexTokenKind.WORD),
                                                LatexToken(0, 4, "bar", LatexTokenKind.WORD)))))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should parse brackets as text (1)`() {
        val text = "[ ]"
        val tree =
                LatexDocumentSyntax(
                        range(0, 0, 0, 3),
                        listOf(
                                LatexTextSyntax(
                                        range(0, 0, 0, 3),
                                        listOf(
                                                LatexToken(0, 0, "[", LatexTokenKind.BEGIN_OPTIONS),
                                                LatexToken(0, 2, "]", LatexTokenKind.END_OPTIONS)))))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should parse brackets as text (2)`() {
        val text = "]"
        val tree =
                LatexDocumentSyntax(
                        range(0, 0, 0, 1),
                        listOf(
                                LatexTextSyntax(
                                        range(0, 0, 0, 1),
                                        listOf(
                                                LatexToken(0, 0, "]", LatexTokenKind.END_OPTIONS)))))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should ignore unmatched braces`() {
        val text = "} }"
        val tree = LatexDocumentSyntax(range(0, 0, 0, 0), listOf())
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should insert missing braces`() {
        val text = "{"
        val tree =
                LatexDocumentSyntax(
                        range(0, 0, 0, 1),
                        listOf(
                                LatexGroupSyntax(
                                        range(0, 0, 0, 1),
                                        left = LatexToken(0, 0, "{", LatexTokenKind.BEGIN_GROUP),
                                        right = null,
                                        children = listOf())))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should parse nested groups`() {
        val text = "{\n{\n}\n}"
        val tree =
                LatexDocumentSyntax(
                        range(0, 0, 3, 1),
                        listOf(
                                LatexGroupSyntax(
                                        range(0, 0, 3, 1),
                                        left = LatexToken(0, 0, "{", LatexTokenKind.BEGIN_GROUP),
                                        right = LatexToken(3, 0, "}", LatexTokenKind.END_GROUP),
                                        children = listOf(
                                                LatexGroupSyntax(
                                                        range(1, 0, 2, 1),
                                                        left = LatexToken(1, 0, "{", LatexTokenKind.BEGIN_GROUP),
                                                        right = LatexToken(2, 0, "}", LatexTokenKind.END_GROUP),
                                                        children = listOf())))))
        assertEquals(tree, LatexParser.parse(text))
    }
}
