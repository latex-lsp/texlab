package texlab.completion.latex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class LatexUserCommandProviderTests {
    private fun verify(builder: OldWorkspaceBuilder, expected: Array<String>) = runBlocking<Unit> {
        builder.completion("foo.tex", 1, 1)
                .let { LatexUserCommandProvider.get(it) }
                .map { it.label }
                .toTypedArray()
                .also { assertArrayEquals(expected, it) }
    }

    @Test
    fun `it should include commands from related documents`() = runBlocking<Unit> {
        val expected = arrayOf("include", "foo")
        OldWorkspaceBuilder()
                .document("foo.tex", "\\include{bar.tex}\n\\")
                .document("bar.tex", "\\foo")
                .also { verify(it, expected) }
    }

    @Test
    fun `it should not include the current command`() = runBlocking<Unit> {
        val expected = arrayOf("bar")
        OldWorkspaceBuilder()
                .document("foo.tex", "\\bar\n\\baz")
                .also { verify(it, expected) }
    }
}
