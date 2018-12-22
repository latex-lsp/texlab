package texlab.syntax

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.syntax.latex.LatexToken
import texlab.syntax.latex.LatexTokenKind

class TokenTests {

    @Test
    fun `it should provide computed properties`() {
        val token = LatexToken(42, 13, "foo", LatexTokenKind.WORD)
        assertEquals(42, token.line)
        assertEquals(13, token.character)
        assertEquals(Position(42, 16), token.end)
        assertEquals(Range(Position(42, 13), Position(42, 16)), token.range)
    }
}
