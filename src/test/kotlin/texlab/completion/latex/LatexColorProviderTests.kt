package texlab.completion.latex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class LatexColorProviderTests {
    @Test
    fun `it should provide items when inside of a color command`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "\\color{}")
                .completion("foo.tex", 0, 7)
                .let { LatexColorProvider.get(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when not inside of a color command`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "\\foo{}")
                .completion("foo.tex", 0, 5)
                .let { LatexColorProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
