package texlab.completion.latex

import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class LatexColorProviderTests {
    @Test
    fun `it should provide items when inside of a color command`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\color{}")
                .completion("foo.tex", 0, 7)
                .let { LatexColorProvider.complete(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when not inside of a color command`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\foo{}")
                .completion("foo.tex", 0, 5)
                .let { LatexColorProvider.complete(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
