package texlab.completion

import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import texlab.completion.latex.LatexColorProvider

class LatexColorProviderTests {
    @Test
    fun `it should provide items when inside of a color command`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\color{}")
                .completion("foo.tex", 0, 7)
                .let { LatexColorProvider.complete(it) }
                .map { it.label }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when not inside of a color command`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\foo{}")
                .completion("foo.tex", 0, 5)
                .let { LatexColorProvider.complete(it) }
                .map { it.label }
                .also { assertTrue(it.isEmpty()) }
    }
}
