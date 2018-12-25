package texlab.completion

import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import texlab.completion.latex.DefineColorSetModelProvider

class DefineColorSetModelProviderTests {
    @Test
    fun `it should provide items when inside of a color command`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\definecolorset{}")
                .completion("foo.tex", 0, 16)
                .let { DefineColorSetModelProvider.complete(it) }
                .also { Assertions.assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when not inside of a color command`() {
        WorkspaceBuilder()
                .document("foo.tex", "\\definecolorset{}{}")
                .completion("foo.tex", 0, 18)
                .let { DefineColorSetModelProvider.complete(it) }
                .also { Assertions.assertTrue(it.isEmpty()) }
    }
}
