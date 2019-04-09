package texlab.syntax.latex

import io.kotlintest.matchers.boolean.shouldBeFalse
import io.kotlintest.matchers.boolean.shouldBeTrue
import io.kotlintest.matchers.collections.shouldHaveSize
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec

class LatexIncludeTests : StringSpec({
    "find should handle valid includes" {
        val root = LatexParser.parse("\\usepackage{foo}\n\\include{bar baz}")
        val includes = LatexInclude.find(root)
        includes.shouldHaveSize(2)
        includes[0].path.shouldBe("foo")
        includes[0].isUnitImport.shouldBeTrue()
        includes[1].path.shouldBe("bar baz")
        includes[1].isUnitImport.shouldBeFalse()
    }

    "find should handle invalid includes" {
        val root = LatexParser.parse("\\documentclass\n\\input{}")
        val includes = LatexInclude.find(root)
        includes.shouldHaveSize(0)
    }
})