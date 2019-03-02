package texlab.completion.latex.data.symbols

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import java.nio.file.Paths

class LatexArgumentSymbolProviderTests {
    @Test
    fun `it should provide arguments for commands`() = runBlocking {
        val argument1 = LatexArgumentSymbol("\\mathbb", "amsmath.sty", "A", 0, 0)
        val argument2 = LatexArgumentSymbol("\\mathbb", "amsmath.sty", "B", 0, 1)
        val argument3 = LatexArgumentSymbol("\\mathbb", "amsmath.sty", "C", 0, 2)
        val index = LatexSymbolIndex(emptyList(), listOf(argument1, argument2, argument3))
        val directory = Paths.get("symbols")
        val database = LatexSymbolDatabase(index, directory)
        val provider = LatexArgumentSymbolProvider(database)

        val items = WorkspaceBuilder()
                .document("foo.tex", "\\usepackage{amsmath}\n\\mathbb{}")
                .completion("foo.tex", 1, 8)
                .let { provider.get(it) }

        assertEquals(3, items.size)
        assertEquals("A", items[0].label)
        assertEquals("B", items[1].label)
        assertEquals("C", items[2].label)
    }
}
