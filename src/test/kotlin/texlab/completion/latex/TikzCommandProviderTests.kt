package texlab.completion.latex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder
import texlab.completion.latex.data.LatexComponent
import texlab.completion.latex.data.LatexComponentSource

class TikzCommandProviderTests {
    private val database = object : LatexComponentSource {
        override fun getComponent(fileName: String): LatexComponent? {
            return LatexComponent(listOf(fileName), emptyList(), emptyList(), emptyList())
        }
    }

    private val provider = TikzCommandProvider(database)

    @Test
    fun `it should provide commands when TikZ is included`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "\\usepackage{tikz}\n\\")
                .completion("foo.tex", 1, 1)
                .let { provider.get(it) }
                .also { assertTrue(it.isNotEmpty()) }
    }

    @Test
    fun `it should not provide commands when TikZ is not included`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "\\")
                .completion("foo.tex", 0, 1)
                .let { provider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
