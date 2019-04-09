package texlab.syntax.latex

import io.kotlintest.matchers.collections.shouldBeEmpty
import io.kotlintest.matchers.collections.shouldHaveSize
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec

class LatexCitationTests : StringSpec({
    "find should handle valid citations" {
        val text = "\\cite{foo}"
        val root = LatexParser.parse(text)
        val citations = LatexCitation.find(root)
        citations.shouldHaveSize(1)
        citations[0].name.text.shouldBe("foo")
    }

    "find should handle invalid citations" {
        val text = "\\cite{}\n\\cite{foo bar}\n\\cite{\\foo}"
        val root = LatexParser.parse(text)
        val citations = LatexCitation.find(root)
        citations.shouldBeEmpty()
    }
})
