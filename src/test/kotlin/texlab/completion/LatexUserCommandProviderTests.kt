package texlab.completion

import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import texlab.completion.latex.LatexUserCommandProvider

class LatexUserCommandProviderTests {
    private fun verify(builder: WorkspaceBuilder, expected: Array<String>) {
        builder.completion("foo.tex", 1, 1)
                .let { LatexUserCommandProvider.complete(it) }
                .map { it.label }
                .toTypedArray()
                .also { assertArrayEquals(expected, it) }
    }

    @Test
    fun `it should include commands from related documents`() {
        val expected = arrayOf("include", "foo")
        WorkspaceBuilder()
                .document("foo.tex", "\\include{bar.tex}\n\\")
                .document("bar.tex", "\\foo")
                .also { verify(it, expected) }
    }

    @Test
    fun `it should not include the current command`() {
        val expected = arrayOf("bar")
        WorkspaceBuilder()
                .document("foo.tex", "\\bar\n\\baz")
                .also { verify(it, expected) }
    }
}
