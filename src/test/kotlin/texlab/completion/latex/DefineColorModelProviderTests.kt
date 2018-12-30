package texlab.completion.latex

import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class DefineColorModelProviderTests {
    @Test
    fun `it should provide items when inside of a color command`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\definecolor{foo}{}")
                .completion("foo.tex", 0, 18)
                .let { DefineColorModelProvider.complete(it) }
                .also { assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when not inside of a color command`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\definecolor{}{}")
                .completion("foo.tex", 0, 13)
                .let { DefineColorModelProvider.complete(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
