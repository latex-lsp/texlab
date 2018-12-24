package texlab.completion.latex

import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.Language
import texlab.completion.CompletionTestsHelper

class DefineColorModelProviderTests {

    private fun verify(language: Language, text: String, line: Int, character: Int): Boolean {
        val workspace = CompletionTestsHelper.createWorkspace(language to text)
        val request = CompletionTestsHelper.createRequest(workspace, 0, line, character)
        val provider = DefineColorModelProvider()
        return provider.getItems(request).isNotEmpty()
    }

    @Test
    fun `it should provide completion in LaTeX documents`() {
        assertTrue(verify(Language.LATEX, "\\definecolor{name}\n{}{spec}", 1, 1))
    }

    @Test
    fun `it should provide not provide completion in BibTeX documents`() {
        assertFalse(verify(Language.BIBTEX, "\\definecolor{name}\n{}{spec}", 1, 1))
    }

    @Test
    fun `it should not provide completion outside of commands`() {
        assertFalse(verify(Language.LATEX, "\\definecolor{name}\n{}{spec}", 1, 3))
    }
}
