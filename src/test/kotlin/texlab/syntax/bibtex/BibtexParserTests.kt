package texlab.syntax.bibtex

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

class BibtexParserTests {
    @ParameterizedTest
    @ValueSource(ints = [1, 2, 3, 4])
    fun `parse and show should be inverses of each other`(index: Int) {
        val text = javaClass.getResourceAsStream("Example$index.bib")
                .readBytes()
                .toString(Charsets.UTF_8)

        val tree1 = BibtexParser.parse(text)
        val tree2 = BibtexParser.parse(BibtexPrinter.print(tree1))
        assertEquals(tree1, tree2)
    }
}
