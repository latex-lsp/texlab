import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import texlab.syntax.latex.*

class LatexParserTests : StringSpec({
    "it should parse the empty document" {
        val text = ""
        val tree = LatexDocumentSyntax(emptyList())
        LatexParser.parse(text).shouldBe(tree)
    }

    "it should parse commands without arguments and options" {
        val text = "\\foo"
        val tree =
                LatexDocumentSyntax(
                        listOf(
                                LatexCommandSyntax(
                                        name = LatexToken(0, 0, "\\foo", LatexTokenKind.COMMAND),
                                        options = null,
                                        args = emptyList())))
        LatexParser.parse(text).shouldBe(tree)
    }

    "it should parse commands with options" {
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
        LatexParser.parse(text).shouldBe(tree)
    }

    "it should parse commands with empty arguments" {
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
        LatexParser.parse(text).shouldBe(tree)
    }

    "it should parse commands with text arguments" {
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
        LatexParser.parse(text).shouldBe(tree)
    }

    "it should parse text" {
        val text = "foo bar"
        val tree =
                LatexDocumentSyntax(
                        listOf(
                                LatexTextSyntax(
                                        listOf(
                                                LatexToken(0, 0, "foo", LatexTokenKind.WORD),
                                                LatexToken(0, 4, "bar", LatexTokenKind.WORD)))))
        LatexParser.parse(text).shouldBe(tree)
    }

    "it should parse brackets as text (1)" {
        val text = "[ ]"
        val tree =
                LatexDocumentSyntax(
                        listOf(
                                LatexTextSyntax(
                                        listOf(
                                                LatexToken(0, 0, "[", LatexTokenKind.BEGIN_OPTIONS),
                                                LatexToken(0, 2, "]", LatexTokenKind.END_OPTIONS)))))
        LatexParser.parse(text).shouldBe(tree)
    }

    "it should parse brackets as text (2)" {
        val text = "]"
        val tree =
                LatexDocumentSyntax(
                        listOf(
                                LatexTextSyntax(
                                        listOf(
                                                LatexToken(0, 0, "]", LatexTokenKind.END_OPTIONS)))))
        LatexParser.parse(text).shouldBe(tree)
    }

    "it should ignore unmatched braces" {
        val text = "} }"
        val tree = LatexDocumentSyntax(emptyList())
        LatexParser.parse(text).shouldBe(tree)
    }

    "it should insert missing braces" {
        val text = "{"
        val tree =
                LatexDocumentSyntax(
                        listOf(
                                LatexGroupSyntax(
                                        left = LatexToken(0, 0, "{", LatexTokenKind.BEGIN_GROUP),
                                        right = null,
                                        children = emptyList())))
        LatexParser.parse(text).shouldBe(tree)
    }

    "it should parse nested groups" {
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
        LatexParser.parse(text).shouldBe(tree)
    }
})