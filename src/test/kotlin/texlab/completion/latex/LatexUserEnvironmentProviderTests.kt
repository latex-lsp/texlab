package texlab.completion.latex

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.CompletionParams
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder
import texlab.provider.FeatureRequest

class LatexUserEnvironmentProviderTests {
    private fun verify(request: FeatureRequest<CompletionParams>, expected: Array<String>) = runBlocking<Unit> {
        LatexUserEnvironmentProvider.get(request)
                .map { it.label }
                .toTypedArray()
                .also { assertArrayEquals(expected, it) }
    }

    @Test
    fun `it should include environments from related documents`() = runBlocking<Unit> {
        val expected = arrayOf("foo")
        OldWorkspaceBuilder()
                .document("foo.tex", "\\include{bar.tex}\n\\begin{}")
                .document("bar.tex", "\\begin{foo}\\end{foo}")
                .completion("foo.tex", 1, 7)
                .also { verify(it, expected) }
    }

    @Test
    fun `it should not include the current environment`() = runBlocking<Unit> {
        val expected = arrayOf<String>()
        OldWorkspaceBuilder()
                .document("foo.tex", "\\begin{}")
                .completion("foo.tex", 0, 7)
                .also { verify(it, expected) }
    }
}
