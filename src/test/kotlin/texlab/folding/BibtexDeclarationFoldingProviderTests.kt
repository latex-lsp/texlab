package texlab.folding

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.FoldingRangeKind
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class BibtexDeclarationFoldingProviderTests {
    @Test
    fun `it should provide foldings for entries`() = runBlocking {
        val foldings = OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar = baz\n}")
                .folding("foo.bib")
                .let { BibtexDeclarationFoldingProvider.get(it) }

        assertEquals(1, foldings.size)
        val folding = foldings[0]
        assertEquals(0, folding.startLine)
        assertEquals(0, folding.startCharacter)
        assertEquals(1, folding.endLine)
        assertEquals(0, folding.endCharacter)
        assertEquals(FoldingRangeKind.Region, folding.kind)
    }

    @Test
    fun `it should provide foldings for strings`() = runBlocking {
        val foldings = OldWorkspaceBuilder()
                .document("foo.bib", "@string{foo = \"bar\"}")
                .folding("foo.bib")
                .let { BibtexDeclarationFoldingProvider.get(it) }

        assertEquals(1, foldings.size)
        val folding = foldings[0]
        assertEquals(0, folding.startLine)
        assertEquals(0, folding.startCharacter)
        assertEquals(0, folding.endLine)
        assertEquals(19, folding.endCharacter)
        assertEquals(FoldingRangeKind.Region, folding.kind)
    }

    @Test
    fun `it should not provide foldings for LaTeX documents`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "@article{foo, }")
                .folding("foo.tex")
                .let { BibtexDeclarationFoldingProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
