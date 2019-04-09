package texlab.syntax.latex

import io.kotlintest.matchers.collections.shouldBeEmpty
import io.kotlintest.matchers.collections.shouldHaveSize
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec

class LatexLabelTests : StringSpec({
    "findDefinitions should handle valid labels" {
        val root = LatexParser.parse("\\label{foo}")
        val labels = LatexLabel.findDefinitions(root)
        labels.shouldHaveSize(1)
        labels[0].name.text.shouldBe("foo")
    }

    "findDefinitions should handle invalid labels" {
        val root = LatexParser.parse("\\label{}\n\\label")
        val labels = LatexLabel.findDefinitions(root)
        labels.shouldBeEmpty()
    }

    "findReferences should handle valid labels" {
        val root = LatexParser.parse("\\ref{foo}")
        val labels = LatexLabel.findReferences(root)
        labels.shouldHaveSize(1)
        labels[0].name.text.shouldBe("foo")
    }

    "findReferences should handle invalid labels" {
        val root = LatexParser.parse("\\ref{}\n\\ref")
        val labels = LatexLabel.findReferences(root)
        labels.shouldBeEmpty()
    }
})