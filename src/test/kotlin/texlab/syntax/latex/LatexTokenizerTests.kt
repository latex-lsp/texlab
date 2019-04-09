package texlab.syntax.latex

import io.kotlintest.matchers.types.shouldBeNull
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec

class LatexTokenizerTests : StringSpec({
    fun verify(tokenizer: LatexTokenizer,
               line: Int,
               character: Int,
               text: String,
               kind: LatexTokenKind) {
        val token = LatexToken(line, character, text, kind)
        tokenizer.next().shouldBe(token)
    }

    "it should be able to tokenize words" {
        val tokenizer = LatexTokenizer("foo bar baz")
        verify(tokenizer, 0, 0, "foo", LatexTokenKind.WORD)
        verify(tokenizer, 0, 4, "bar", LatexTokenKind.WORD)
        verify(tokenizer, 0, 8, "baz", LatexTokenKind.WORD)
        tokenizer.next().shouldBeNull()
    }

    "it should be able to tokenize commands" {
        val tokenizer = LatexTokenizer("\\foo\\bar@baz")
        verify(tokenizer, 0, 0, "\\foo", LatexTokenKind.COMMAND)
        verify(tokenizer, 0, 4, "\\bar@baz", LatexTokenKind.COMMAND)
        tokenizer.next().shouldBeNull()
    }

    "it should be able to tokenize escape sequences" {
        val tokenizer = LatexTokenizer("\\foo*\n\\%\\**")
        verify(tokenizer, 0, 0, "\\foo*", LatexTokenKind.COMMAND)
        verify(tokenizer, 1, 0, "\\%", LatexTokenKind.COMMAND)
        verify(tokenizer, 1, 2, "\\*", LatexTokenKind.COMMAND)
        verify(tokenizer, 1, 4, "*", LatexTokenKind.WORD)
        tokenizer.next().shouldBeNull()
    }

    "it should be able to tokenize group delimiters" {
        val tokenizer = LatexTokenizer("{}[]")
        verify(tokenizer, 0, 0, "{", LatexTokenKind.BEGIN_GROUP)
        verify(tokenizer, 0, 1, "}", LatexTokenKind.END_GROUP)
        verify(tokenizer, 0, 2, "[", LatexTokenKind.BEGIN_OPTIONS)
        verify(tokenizer, 0, 3, "]", LatexTokenKind.END_OPTIONS)
        tokenizer.next().shouldBeNull()
    }

    "it should ignore line comments" {
        val tokenizer = LatexTokenizer(" %foo \nfoo")
        verify(tokenizer, 1, 0, "foo", LatexTokenKind.WORD)
        tokenizer.next().shouldBeNull()
    }

    "it should be able to tokenize star commands" {
        val tokenizer = LatexTokenizer("\\foo*")
        verify(tokenizer, 0, 0, "\\foo*", LatexTokenKind.COMMAND)
        tokenizer.next().shouldBeNull()
    }

    "it should be able to tokenize math delimiters" {
        val tokenizer = LatexTokenizer("$$ $ $")
        verify(tokenizer, 0, 0, "$$", LatexTokenKind.MATH)
        verify(tokenizer, 0, 3, "$", LatexTokenKind.MATH)
        verify(tokenizer, 0, 5, "$", LatexTokenKind.MATH)
    }
})