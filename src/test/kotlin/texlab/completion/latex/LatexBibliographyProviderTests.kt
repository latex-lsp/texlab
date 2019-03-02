package texlab.completion.latex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class LatexBibliographyProviderTests {
    @Test
    fun `it should exclude files that are already included`() = runBlocking {
        val builder = WorkspaceBuilder()
                .document("foo.tex", "\\bibliography{bar.bib}\n\\bibliography{}")
                .document("bar.bib", "")
                .document("baz.bib", "")

        val provider = LatexBibliographyProvider()

        val expected = arrayOf("baz.bib")
        val actual = provider
                .get(builder.completion("foo.tex", 1, 14))
                .map { it.label }
                .toTypedArray()
        Assertions.assertArrayEquals(expected, actual)
    }
}
