package texlab.syntax.latex

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test

class LatexTokenizerTests {
    private fun LatexTokenizer.verify(
            line: Int,
            character: Int,
            text: String,
            kind: LatexTokenKind) {
        val expected = LatexToken(line, character, text, kind)
        assertEquals(expected, next())
    }

    @Test
    fun `it should be able to tokenize words`() {
        val tokenizer = LatexTokenizer("foo bar baz")
        tokenizer.verify(0, 0, "foo", LatexTokenKind.WORD)
        tokenizer.verify(0, 4, "bar", LatexTokenKind.WORD)
        tokenizer.verify(0, 8, "baz", LatexTokenKind.WORD)
        assertNull(tokenizer.next())
    }

    @Test
    fun `it should be able to tokenize commands`() {
        val tokenizer = LatexTokenizer("\\foo\\bar@baz")
        tokenizer.verify(0, 0, "\\foo", LatexTokenKind.COMMAND)
        tokenizer.verify(0, 4, "\\bar@baz", LatexTokenKind.COMMAND)
        assertNull(tokenizer.next())
    }

    @Test
    fun `it should be able to parse escape sequences`() {
        val tokenizer = LatexTokenizer("\\foo*\n\\%\\**")
        tokenizer.verify(0, 0, "\\foo*", LatexTokenKind.COMMAND)
        tokenizer.verify(1, 0, "\\%", LatexTokenKind.COMMAND)
        tokenizer.verify(1, 2, "\\*", LatexTokenKind.COMMAND)
        tokenizer.verify(1, 4, "*", LatexTokenKind.WORD)
        assertNull(tokenizer.next())
    }

    @Test
    fun `it should be able to parse group delimiters`() {
        val tokenizer = LatexTokenizer("{}[]")
        tokenizer.verify(0, 0, "{", LatexTokenKind.BEGIN_GROUP)
        tokenizer.verify(0, 1, "}", LatexTokenKind.END_GROUP)
        tokenizer.verify(0, 2, "[", LatexTokenKind.BEGIN_OPTIONS)
        tokenizer.verify(0, 3, "]", LatexTokenKind.END_OPTIONS)
        assertNull(tokenizer.next())
    }

    @Test
    fun `it should be able to ignore line comments`() {
        val tokenizer = LatexTokenizer(" %foo \nfoo")
        tokenizer.verify(1, 0, "foo", LatexTokenKind.WORD)
        assertNull(tokenizer.next())
    }

    @Test
    fun `it should be able to read star commands`() {
        val tokenizer = LatexTokenizer("\\foo*")
        tokenizer.verify(0, 0, "\\foo*", LatexTokenKind.COMMAND)
        assertNull(tokenizer.next())
    }
}
