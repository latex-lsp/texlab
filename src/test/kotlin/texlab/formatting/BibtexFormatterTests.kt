package texlab.formatting

import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import texlab.syntax.bibtex.BibtexDeclarationSyntax
import texlab.syntax.bibtex.BibtexParser

class BibtexFormatterTests : StringSpec({
    fun verify(source: String, expected: String, lineLength: Int = 30) {
        val entry = BibtexParser.parse(source)
                .children
                .filterIsInstance<BibtexDeclarationSyntax>()
                .first()

        val formatter = BibtexFormatter(true, 4, lineLength)
        formatter.format(entry)
                .replace(System.lineSeparator(), "\n")
                .shouldBe(expected)
    }

    "it should wrap long lines" {
        val source = "@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}"
        val expected = """
            @article{foo,
                bar = {Lorem ipsum dolor
                       sit amet,
                       consectetur
                       adipiscing elit.},
            }
        """.trimIndent()
        verify(source, expected)
    }

    "it should not wrap long lines with a line length of zero" {
        val source = "@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}"
        val expected = """
            @article{foo,
                bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit.},
            }
        """.trimIndent()
        verify(source, expected, 0)
    }

    "it should insert trailing commas" {
        val source = "@article{foo, bar = baz}"
        val expected = """
            @article{foo,
                bar = baz,
            }
        """.trimIndent()
        verify(source, expected)
    }

    "it should insert missing braces" {
        val source = "@article{foo, bar = baz,"
        val expected = """
            @article{foo,
                bar = baz,
            }
        """.trimIndent()
        verify(source, expected)
    }

    "it should handle commands" {
        val source = "@article{foo, bar = \"\\baz\",}"
        val expected = """
            @article{foo,
                bar = "\baz",
            }
        """.trimIndent()
        verify(source, expected)
    }

    "it should handle string concatenation" {
        val source = "@article{foo, bar = \"baz\" # \"qux\"}"
        val expected = """
            @article{foo,
                bar = "baz" # "qux",
            }
        """.trimIndent()
        verify(source, expected)
    }

    "it should replace parentheses with braces" {
        val source = "@article(foo,)"
        val expected = """
            @article{foo,
            }
        """.trimIndent()
        verify(source, expected)
    }

    "it should handle strings" {
        val source = "@string{foo=\"bar\"}"
        val expected = """@string{foo = "bar"}"""
        verify(source, expected)
    }

    "it should handle preambles" {
        val source = "@preamble{\n\"foo bar baz\"}"
        val expected = "@preamble{\"foo bar baz\"}"
        verify(source, expected)
    }
})
