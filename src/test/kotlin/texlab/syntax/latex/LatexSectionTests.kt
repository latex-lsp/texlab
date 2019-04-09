package texlab.syntax.latex

import io.kotlintest.matchers.collections.shouldBeEmpty
import io.kotlintest.matchers.collections.shouldHaveSize
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec

class LatexSectionTests : StringSpec({
    "find should handle valid sections" {
        val root = LatexParser.parse("\\section{foo}\n\\chapter{bar baz}")
        val sections = LatexSection.find(root)
        sections.shouldHaveSize(2)
        sections[0].text.shouldBe("foo")
        sections[0].level.shouldBe(1)
        sections[1].text.shouldBe("bar baz")
        sections[1].level.shouldBe(0)
    }

    "find should handle invalid sections" {
        val root = LatexParser.parse("\\paragraph\\subsection{}")
        val sections = LatexSection.find(root)
        sections.shouldBeEmpty()
    }
})