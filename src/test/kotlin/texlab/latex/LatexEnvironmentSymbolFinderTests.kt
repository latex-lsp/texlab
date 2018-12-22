package texlab.latex

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class LatexEnvironmentSymbolFinderTests {

    @Test
    fun `it should find empty and nonempty environments`() {
        val text = "\\begin{foo}\n\\end{}"
        val tree = LatexSyntaxTree(text)
        val range1 = Range(Position(0, 7), Position(0, 10))
        val range2 = Range(Position(1, 5), Position(1, 5))
        val symbol1 = DocumentSymbol("foo", LatexEnvironmentSymbolFinder.kind, range1, range1)
        val symbol2 = DocumentSymbol("", LatexEnvironmentSymbolFinder.kind, range2, range2)
        val symbols = LatexEnvironmentSymbolFinder.find(tree)
        assertEquals(2, symbols.size)
        assertEquals(symbol1, symbols[0])
        assertEquals(symbol2, symbols[1])
    }
}
