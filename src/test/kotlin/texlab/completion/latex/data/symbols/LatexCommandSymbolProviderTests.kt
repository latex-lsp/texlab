package texlab.completion.latex.data.symbols

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import java.nio.file.Paths

class LatexCommandSymbolProviderTests {
    @Test
    fun `it should only provide included symbols`() = runBlocking {
        val command1 = LatexCommandSymbol("\\varDelta", "amsmath.sty", 0)
        val command2 = LatexCommandSymbol("\\LinearACCCII", "linearA.sty", 1)
        val command3 = LatexCommandSymbol("\\varepsilon", null, 2)
        val index = LatexSymbolIndex(listOf(command1, command2, command3), emptyList())
        val directory = Paths.get("symbols")
        val database = LatexSymbolDatabase(index, directory)
        val provider = LatexCommandSymbolProvider(database)

        val items = WorkspaceBuilder()
                .document("foo.tex", "\\usepackage{amsmath}\n\\")
                .completion("foo.tex", 1, 1)
                .let { provider.get(it) }

        assertEquals(2, items.size)
        assertEquals("varDelta", items[0].label)
        assertEquals("varepsilon", items[1].label)
    }
}
