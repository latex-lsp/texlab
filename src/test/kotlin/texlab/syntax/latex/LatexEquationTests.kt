package texlab.syntax.latex

import io.kotlintest.matchers.collections.shouldBeEmpty
import io.kotlintest.matchers.collections.shouldHaveSize
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import texlab.range

class LatexEquationTests : StringSpec({
    "find should handle valid equations" {
        val root = LatexParser.parse("\\[ \\foo \\]")
        val equations = LatexEquation.find(root)
        equations.shouldHaveSize(1)
        equations[0].begin.name.text.shouldBe("\\[")
        equations[0].end.name.text.shouldBe("\\]")
        equations[0].range.shouldBe(range(0, 0, 0, 10))
    }

    "find should handle invalid equations" {
        val root = LatexParser.parse("\\] \\[")
        val equations = LatexEquation.find(root)
        equations.shouldBeEmpty()
    }
})