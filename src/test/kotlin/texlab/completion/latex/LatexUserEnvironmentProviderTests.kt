package texlab.completion.latex

import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import texlab.completion.CompletionRequest

class LatexUserEnvironmentProviderTests {
    private fun verify(request: CompletionRequest, expected: Array<String>) {
        LatexUserEnvironmentProvider.complete(request)
                .map { it.label }
                .toTypedArray()
                .also { assertArrayEquals(expected, it) }
    }

    @Test
    fun `it should include environments from related documents`() {
        val expected = arrayOf("foo")
        WorkspaceBuilder()
                .document("foo.tex", "\\include{bar.tex}\n\\begin{}")
                .document("bar.tex", "\\begin{foo}\\end{foo}")
                .completion("foo.tex", 1, 7)
                .also { verify(it, expected) }
    }

    @Test
    fun `it should not include the current environment`() {
        val expected = arrayOf<String>()
        WorkspaceBuilder()
                .document("foo.tex", "\\begin{}")
                .completion("foo.tex", 0, 7)
                .also { verify(it, expected) }
    }
}
