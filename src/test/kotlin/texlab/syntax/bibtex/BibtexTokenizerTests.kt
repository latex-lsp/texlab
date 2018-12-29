package texlab.syntax.bibtex

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test

class BibtexTokenizerTests {
    private fun BibtexTokenizer.verify(
            line: Int,
            character: Int,
            text: String,
            kind: BibtexTokenKind) {
        val expected = BibtexToken(line, character, text, kind)
        assertEquals(expected, next())
    }

    @Test
    fun `it should be able to tokenize words`() {
        val tokenizer = BibtexTokenizer("foo bar baz")
        tokenizer.verify(0, 0, "foo", BibtexTokenKind.WORD)
        tokenizer.verify(0, 4, "bar", BibtexTokenKind.WORD)
        tokenizer.verify(0, 8, "baz", BibtexTokenKind.WORD)
        assertNull(tokenizer.next())
    }

    @Test
    fun `it should be able to tokenize commands`() {
        val tokenizer = BibtexTokenizer("\\foo\\bar@baz")
        tokenizer.verify(0, 0, "\\foo", BibtexTokenKind.COMMAND)
        tokenizer.verify(0, 4, "\\bar@baz", BibtexTokenKind.COMMAND)
        assertNull(tokenizer.next())
    }

    @Test
    fun `it should be able to parse escape sequences`() {
        val tokenizer = BibtexTokenizer("\\foo*\n\\%\\**")
        tokenizer.verify(0, 0, "\\foo*", BibtexTokenKind.COMMAND)
        tokenizer.verify(1, 0, "\\%", BibtexTokenKind.COMMAND)
        tokenizer.verify(1, 2, "\\*", BibtexTokenKind.COMMAND)
        tokenizer.verify(1, 4, "*", BibtexTokenKind.WORD)
        assertNull(tokenizer.next())
    }

    @Test
    fun `it should be able to parse delimiters`() {
        val tokenizer = BibtexTokenizer("{}()\"")
        tokenizer.verify(0, 0, "{", BibtexTokenKind.BEGIN_BRACE)
        tokenizer.verify(0, 1, "}", BibtexTokenKind.END_BRACE)
        tokenizer.verify(0, 2, "(", BibtexTokenKind.BEGIN_PAREN)
        tokenizer.verify(0, 3, ")", BibtexTokenKind.END_PAREN)
        tokenizer.verify(0, 4, "\"", BibtexTokenKind.QUOTE)
        assertNull(tokenizer.next())
    }

    @Test
    fun `it should be able to parse types`() {
        val tokenizer = BibtexTokenizer("@pReAmBlE\n@article\n@string")
        tokenizer.verify(0, 0, "@pReAmBlE", BibtexTokenKind.PREAMBLE_TYPE)
        tokenizer.verify(1, 0, "@article", BibtexTokenKind.ENTRY_TYPE)
        tokenizer.verify(2, 0, "@string", BibtexTokenKind.STRING_TYPE)
        assertNull(tokenizer.next())
    }

    @Test
    fun `it should be able to parse operators`() {
        val tokenizer = BibtexTokenizer("=,#")
        tokenizer.verify(0, 0, "=", BibtexTokenKind.ASSIGN)
        tokenizer.verify(0, 1, ",", BibtexTokenKind.COMMA)
        tokenizer.verify(0, 2, "#", BibtexTokenKind.CONCAT)
        assertNull(tokenizer.next())
    }
}
