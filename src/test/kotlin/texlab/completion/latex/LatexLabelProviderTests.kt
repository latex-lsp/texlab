package texlab.completion.latex

import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.Language
import texlab.completion.CompletionTestsHelper

class LatexLabelProviderTests {

    private val provider = LatexLabelProvider()

    @Test
    fun `it should find labels defined in the same file`() {
        val workspace = CompletionTestsHelper.createWorkspace(Language.LATEX to "\\label{foo}\\ref\n{}")
        val request = CompletionTestsHelper.createRequest(workspace, 0, 1, 1)
        val actual = provider.getItems(request).map { it.label }.toTypedArray()
        assertArrayEquals(arrayOf("foo"), actual)
    }
}
