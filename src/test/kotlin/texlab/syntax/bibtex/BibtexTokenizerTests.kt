package texlab.syntax.bibtex

import io.kotlintest.matchers.types.shouldBeNull
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec

class BibtexTokenizerTests : StringSpec({
    fun verify(tokenizer: BibtexTokenizer,
               line: Int,
               character: Int,
               text: String,
               kind: BibtexTokenKind) {
        val token = BibtexToken(line, character, text, kind)
        tokenizer.next().shouldBe(token)
    }

    "it should be able to tokenize words" {
        val tokenizer = BibtexTokenizer("foo bar baz")
        verify(tokenizer, 0, 0, "foo", BibtexTokenKind.WORD)
        verify(tokenizer, 0, 4, "bar", BibtexTokenKind.WORD)
        verify(tokenizer, 0, 8, "baz", BibtexTokenKind.WORD)
    }

    "it should be able to tokenize commands" {
        val tokenizer = BibtexTokenizer("\\foo\\bar@baz")
        verify(tokenizer, 0, 0, "\\foo", BibtexTokenKind.COMMAND)
        verify(tokenizer, 0, 4, "\\bar@baz", BibtexTokenKind.COMMAND)
        tokenizer.next().shouldBeNull()
    }

    "it should be able to parse escape sequences" {
        val tokenizer = BibtexTokenizer("\\foo*\n\\%\\**")
        verify(tokenizer, 0, 0, "\\foo*", BibtexTokenKind.COMMAND)
        verify(tokenizer, 1, 0, "\\%", BibtexTokenKind.COMMAND)
        verify(tokenizer, 1, 2, "\\*", BibtexTokenKind.COMMAND)
        verify(tokenizer, 1, 4, "*", BibtexTokenKind.WORD)
        tokenizer.next().shouldBeNull()
    }

    "it should be able to parse delimiters" {
        val tokenizer = BibtexTokenizer("{}()\"")
        verify(tokenizer, 0, 0, "{", BibtexTokenKind.BEGIN_BRACE)
        verify(tokenizer, 0, 1, "}", BibtexTokenKind.END_BRACE)
        verify(tokenizer, 0, 2, "(", BibtexTokenKind.BEGIN_PAREN)
        verify(tokenizer, 0, 3, ")", BibtexTokenKind.END_PAREN)
        verify(tokenizer, 0, 4, "\"", BibtexTokenKind.QUOTE)
        tokenizer.next().shouldBeNull()
    }

    "it should be able to parse types" {
        val tokenizer = BibtexTokenizer("@pReAmBlE\n@article\n@string")
        verify(tokenizer, 0, 0, "@pReAmBlE", BibtexTokenKind.PREAMBLE_TYPE)
        verify(tokenizer, 1, 0, "@article", BibtexTokenKind.ENTRY_TYPE)
        verify(tokenizer, 2, 0, "@string", BibtexTokenKind.STRING_TYPE)
        tokenizer.next().shouldBeNull()
    }

    "it should be able to parse operators" {
        val tokenizer = BibtexTokenizer("=,#")
        verify(tokenizer, 0, 0, "=", BibtexTokenKind.ASSIGN)
        verify(tokenizer, 0, 1, ",", BibtexTokenKind.COMMA)
        verify(tokenizer, 0, 2, "#", BibtexTokenKind.CONCAT)
        tokenizer.next().shouldBeNull()
    }
})