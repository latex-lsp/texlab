package texlab.completion.latex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class LatexKernelEnvironmentProviderTests {
    @Test
    fun `it should provide items when inside of an environment delimiter`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "\\begin{}")
                .completion("foo.tex", 0, 7)
                .let { LatexKernelEnvironmentProvider.get(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when not inside of an environment delimiter`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "\\foo{}")
                .completion("foo.tex", 0, 5)
                .let { LatexKernelEnvironmentProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
