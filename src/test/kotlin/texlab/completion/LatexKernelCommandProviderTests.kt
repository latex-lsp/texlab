package texlab.completion

import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import texlab.completion.latex.LatexKernelCommandProvider

class LatexKernelCommandProviderTests {
    @Test
    fun `it should provide items when inside of a command`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\foo")
                .completion("foo.tex", 0, 2)
                .let { LatexKernelCommandProvider.complete(it) }
                .also { Assertions.assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when not inside of a command`() {
        WorkspaceBuilder()
                .document("foo.tex", "")
                .completion("foo.tex", 0, 0)
                .let { LatexKernelCommandProvider.complete(it) }
                .also { Assertions.assertTrue(it.isEmpty()) }
    }
}
