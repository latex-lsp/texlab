package texlab.completion.latex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class LatexIncludeProviderTests {
    @Test
    fun `it should exclude files that are already included`() = runBlocking {
        val builder = WorkspaceBuilder()
                .document("foo.tex", "\\include{bar}\n\\include{}")
                .document("bar.tex", "")
                .document("baz.tex", "")

        val provider = LatexIncludeProvider()

        val expected = arrayOf("baz")
        val actual = provider
                .get(builder.completion("foo.tex", 1, 9))
                .map { it.label }
                .toTypedArray()
        assertArrayEquals(expected, actual)
    }
}

