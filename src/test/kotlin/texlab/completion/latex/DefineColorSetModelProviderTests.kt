package texlab.completion.latex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class DefineColorSetModelProviderTests {
    @Test
    fun `it should provide items when inside of a color command`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "\\definecolorset{}")
                .completion("foo.tex", 0, 16)
                .let { DefineColorSetModelProvider.get(it) }
                .also { Assertions.assertFalse(it.isEmpty()) }
    }

    @Test
    fun `it should not provide items when not inside of a color command`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "\\definecolorset{}{}")
                .completion("foo.tex", 0, 18)
                .let { DefineColorSetModelProvider.get(it) }
                .also { Assertions.assertTrue(it.isEmpty()) }
    }
}
