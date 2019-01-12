package texlab.completion.latex

import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class LatexKernelCommandProviderTests {
    @Test
    fun `it should provide items when inside of a command without arguments`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\foo")
                .completion("foo.tex", 0, 2)
                .let { LatexKernelCommandProvider.complete(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should provide items when inside of a command with arguments`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\foo{bar}")
                .completion("foo.tex", 0, 4)
                .let { LatexKernelCommandProvider.complete(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when not inside of a command`() {
        WorkspaceBuilder()
                .document("foo.tex", "")
                .completion("foo.tex", 0, 0)
                .let { LatexKernelCommandProvider.complete(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
