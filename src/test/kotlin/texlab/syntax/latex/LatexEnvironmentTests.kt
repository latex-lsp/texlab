package texlab.syntax.latex

import io.kotlintest.matchers.collections.shouldBeEmpty
import io.kotlintest.matchers.collections.shouldHaveSize
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import texlab.range

class LatexEnvironmentTests : StringSpec({
    "find should handle valid environments" {
        val root = LatexParser.parse("\\begin{foo}\n\\end{bar}")
        val environments = LatexEnvironment.find(root)
        environments.shouldHaveSize(1)
        environments[0].beginName.shouldBe("foo")
        environments[0].beginNameRange.shouldBe(range(0, 7, 0, 10))
        environments[0].endName.shouldBe("bar")
        environments[0].endNameRange.shouldBe(range(1, 5, 1, 8))
        environments[0].range.shouldBe(range(0, 0, 1, 9))
    }

    "find should handle environments with empty names" {
        val root = LatexParser.parse("\\begin{}\\end{}")
        val environments = LatexEnvironment.find(root)
        environments.shouldHaveSize(1)
        environments[0].begin.name.text.shouldBe("\\begin")
        environments[0].beginName.shouldBe("")
        environments[0].beginNameRange.shouldBe(range(0, 7, 0, 7))
        environments[0].end.name.text.shouldBe("\\end")
        environments[0].endName.shouldBe("")
        environments[0].endNameRange.shouldBe(range(0, 13, 0, 13))
        environments[0].range.shouldBe(range(0, 0, 0, 14))
    }

    "find should ignore unmatched delimiters" {
        val root = LatexParser.parse("\\end{foo}\n\\begin{bar}\\end")
        val environments = LatexEnvironment.find(root)
        environments.shouldBeEmpty()
    }
})