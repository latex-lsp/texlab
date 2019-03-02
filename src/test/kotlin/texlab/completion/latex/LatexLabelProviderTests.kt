package texlab.completion.latex

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class LatexLabelProviderTests {
    @Test
    fun `it should complete labels from related documents`() = runBlocking<Unit> {
        val expected = arrayOf("baz", "qux")
        WorkspaceBuilder()
                .document("foo.tex", "\\include{bar.tex}\n\\ref{}")
                .document("bar.tex", "\\label{baz}\n\\label{qux}")
                .completion("foo.tex", 1, 5)
                .let { LatexLabelProvider.get(it) }
                .map { it.label }
                .toTypedArray()
                .also { assertArrayEquals(expected, it) }
    }

    @Test
    fun `it should not complete labels when outside of a command`() = runBlocking<Unit> {
        WorkspaceBuilder()
                .document("foo.tex", "\\label{foo}\n\\foo{}")
                .completion("foo.tex", 1, 5)
                .let { LatexLabelProvider.get(it) }
                .also { assertEquals(0, it.size) }
    }
}
