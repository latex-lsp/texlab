package texlab.completion

import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import texlab.completion.latex.LatexBibliographyProvider

class LatexBibliographyProviderTests {
    @Test
    fun `it should exclude files that are already included`() {
        val builder = WorkspaceBuilder()
                .document("foo.tex", "\\bibliography{bar.bib}\n\\bibliography{}")
                .document("bar.bib", "")
                .document("baz.bib", "")

        val provider = LatexBibliographyProvider(builder.workspace)

        val expected = arrayOf("baz.bib")
        val actual = provider
                .complete(builder.completion("foo.tex", 1, 14))
                .map { it.label }
                .toTypedArray()
        Assertions.assertArrayEquals(expected, actual)
    }
}
