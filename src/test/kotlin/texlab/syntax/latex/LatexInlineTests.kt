package texlab.syntax.latex

import io.kotlintest.matchers.collections.shouldBeEmpty
import io.kotlintest.matchers.collections.shouldHaveSize
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import texlab.range

class LatexInlineTests : StringSpec({
    "find should handle valid inlines" {
        val root = LatexParser.parse("$ x = 1 $\n$ y = 2 $")
        val inlines = LatexInline.find(root)
        inlines.shouldHaveSize(2)
        inlines[0].range.shouldBe(range(0, 0, 0, 9))
        inlines[1].range.shouldBe(range(1, 0, 1, 9))
    }

    "find should ignore invalid inlines" {
        val root = LatexParser.parse("$")
        val inlines = LatexInline.find(root)
        inlines.shouldBeEmpty()
    }
})
