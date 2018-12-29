package texlab.syntax.latex

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class LatexParserTests {
    @Test
    fun `it should parse commands without arguments and options`() {
        val text = "\\foo"
        val tree =
                LatexDocumentSyntax(
                        listOf(
                                LatexCommandSyntax(
                                        name = LatexToken(0, 0, "\\foo", LatexTokenKind.COMMAND),
                                        options = null,
                                        args = emptyList())))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should parse commands with options`() {
        val text = "\\foo[bar]"
        val opts =
                LatexGroupSyntax(
                        left = LatexToken(0, 4, "[", LatexTokenKind.BEGIN_OPTIONS),
                        right = LatexToken(0, 8, "]", LatexTokenKind.END_OPTIONS),
                        children = listOf(LatexTextSyntax(
                                listOf(LatexToken(0, 5, "bar", LatexTokenKind.WORD)))))
        val tree =
                LatexDocumentSyntax(
                        listOf(
                                LatexCommandSyntax(
                                        name = LatexToken(0, 0, "\\foo", LatexTokenKind.COMMAND),
                                        options = opts,
                                        args = emptyList())))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should parse commands with empty arguments`() {
        val text = "\\foo{}"
        val args =
                LatexGroupSyntax(
                        left = LatexToken(0, 4, "{", LatexTokenKind.BEGIN_GROUP),
                        right = LatexToken(0, 5, "}", LatexTokenKind.END_GROUP),
                        children = listOf())
        val tree =
                LatexDocumentSyntax(
                        listOf(
                                LatexCommandSyntax(
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
                        left = LatexToken(0, 6, "{", LatexTokenKind.BEGIN_GROUP),
                        right = LatexToken(0, 10, "}", LatexTokenKind.END_GROUP),
                        children = listOf(
                                LatexTextSyntax(
                                        listOf(LatexToken(0, 7, "foo", LatexTokenKind.WORD))))))
        val tree =
                LatexDocumentSyntax(
                        listOf(
                                LatexCommandSyntax(
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
                        listOf(
                                LatexTextSyntax(
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
                        listOf(
                                LatexTextSyntax(
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
                        listOf(
                                LatexTextSyntax(
                                        listOf(
                                                LatexToken(0, 0, "]", LatexTokenKind.END_OPTIONS)))))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should ignore unmatched braces`() {
        val text = "} }"
        val tree = LatexDocumentSyntax(emptyList())
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should insert missing braces`() {
        val text = "{"
        val tree =
                LatexDocumentSyntax(
                        listOf(
                                LatexGroupSyntax(
                                        left = LatexToken(0, 0, "{", LatexTokenKind.BEGIN_GROUP),
                                        right = null,
                                        children = emptyList())))
        assertEquals(tree, LatexParser.parse(text))
    }

    @Test
    fun `it should parse nested groups`() {
        val text = "{\n{\n}\n}"
        val tree =
                LatexDocumentSyntax(
                        listOf(
                                LatexGroupSyntax(
                                        left = LatexToken(0, 0, "{", LatexTokenKind.BEGIN_GROUP),
                                        right = LatexToken(3, 0, "}", LatexTokenKind.END_GROUP),
                                        children = listOf(
                                                LatexGroupSyntax(
                                                        left = LatexToken(1, 0, "{", LatexTokenKind.BEGIN_GROUP),
                                                        right = LatexToken(2, 0, "}", LatexTokenKind.END_GROUP),
                                                        children = emptyList())))))
        assertEquals(tree, LatexParser.parse(text))
    }
}
