package texlab.syntax

import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import org.eclipse.lsp4j.Position
import texlab.range
import texlab.syntax.latex.LatexToken
import texlab.syntax.latex.LatexTokenKind

class TokenTests : StringSpec({
    "it should provide computed properties" {
        val token = LatexToken(42, 13, "foo", LatexTokenKind.WORD)
        token.line.shouldBe(42)
        token.character.shouldBe(13)
        token.start.shouldBe(Position(42, 13))
        token.end.shouldBe(Position(42, 16))
        token.range.shouldBe(range(42, 13, 42, 16))
    }
})