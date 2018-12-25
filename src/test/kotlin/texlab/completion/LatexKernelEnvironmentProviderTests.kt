package texlab.completion

import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import texlab.completion.latex.LatexKernelEnvironmentProvider

class LatexKernelEnvironmentProviderTests {
    @Test
    fun `it should provide items when inside of an environment delimiter`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\begin{}")
                .completion("foo.tex", 0, 7)
                .let { LatexKernelEnvironmentProvider.complete(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when not inside of an environment delimiter`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\foo{}")
                .completion("foo.tex", 0, 5)
                .let { LatexKernelEnvironmentProvider.complete(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
