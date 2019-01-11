package texlab.formatting

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.syntax.bibtex.BibtexDeclarationSyntax
import texlab.syntax.bibtex.BibtexParser

class BibtexFormatterTests {
    private fun verify(source: String, expected: String) {
        val entry = BibtexParser.parse(source)
                .children
                .filterIsInstance<BibtexDeclarationSyntax>()
                .first()

        val formatter = BibtexFormatter(true, 4, 30)
        val actual = formatter
                .format(entry)
                .replace(System.lineSeparator(), "\n")
        assertEquals(expected, actual)
    }

    @Test
    fun `it should wrap long lines`() {
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

    @Test
    fun `it should insert trailing commas`() {
        val source = "@article{foo, bar = baz}"
        val expected = """
            @article{foo,
                bar = baz,
            }
        """.trimIndent()
        verify(source, expected)
    }

    @Test
    fun `it should insert missing braces`() {
        val source = "@article{foo, bar = baz,"
        val expected = """
            @article{foo,
                bar = baz,
            }
        """.trimIndent()
        verify(source, expected)
    }

    @Test
    fun `it should handle commands`() {
        val source = "@article{foo, bar = \"\\baz\",}"
        val expected = """
            @article{foo,
                bar = "\baz",
            }
        """.trimIndent()
        verify(source, expected)
    }

    @Test
    fun `it should handle string concatenation`() {
        val source = "@article{foo, bar = \"baz\" # \"qux\"}"
        val expected = """
            @article{foo,
                bar = "baz" # "qux",
            }
        """.trimIndent()
        verify(source, expected)
    }

    @Test
    fun `it should replace parens with braces`() {
        val source = "@article(foo,)"
        val expected = """
            @article{foo,
            }
        """.trimIndent()
        verify(source, expected)
    }

    @Test
    fun `it should handle strings`() {
        val source = "@string{foo=\"bar\"}"
        val expected = """@string{foo = "bar"}"""
        verify(source, expected)
    }

    @Test
    fun `it should handle preambles`() {
        val source = "@preamble{\n\"foo bar baz\"}"
        val expected = "@preamble{\"foo bar baz\"}"
        verify(source, expected)
    }
}
