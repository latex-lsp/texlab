package texlab.completion.latex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class LatexKernelCommandProviderTests {
    @Test
    fun `it should provide items when inside of a command without arguments`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "\\foo")
                .completion("foo.tex", 0, 2)
                .let { LatexKernelCommandProvider.get(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should provide items when inside of a command with arguments`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "\\foo{bar}")
                .completion("foo.tex", 0, 4)
                .let { LatexKernelCommandProvider.get(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when not inside of a command`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "")
                .completion("foo.tex", 0, 0)
                .let { LatexKernelCommandProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
