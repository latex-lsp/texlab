package texlab

import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import java.net.URI

class WorkspaceTests {
    private fun relatedDocuments(workspace: Workspace, uri: URI): Array<URI> {
        return workspace
                .relatedDocuments(uri)
                .map { it.uri }
                .toTypedArray()
    }

    @Test
    fun `it should append extensions when analyzing includes`() {
        val builder = WorkspaceBuilder()
        val uri = builder.uri("foo.tex")
        val workspace = builder
                .document(uri, "\\include{bar/baz}")
                .document("bar/baz.tex", "")
                .workspace

        val expected = workspace.documentsByUri.keys.toTypedArray()
        val actual = relatedDocuments(workspace, uri)
        assertArrayEquals(expected, actual)
    }

    @Test
    fun `it should ignore invalid includes`() {
        val builder = WorkspaceBuilder()
        val uri = builder.uri("foo.tex")
        val workspace = builder
                .document(uri, "\\include{<foo>?|bar|:}\n\\include{}")
                .workspace

        val expected = arrayOf(uri)
        val actual = relatedDocuments(workspace, uri)
        assertArrayEquals(expected, actual)
    }

    @Test
    fun `it should find related bibliographies`() {
        val builder = WorkspaceBuilder()
        val uri = builder.uri("foo.tex")
        val workspace = builder
                .document(uri, "\\addbibresource{bar.bib}")
                .document("bar.bib", "")
                .workspace

        val expected = workspace.documentsByUri.keys.toTypedArray()
        val actual = relatedDocuments(workspace, uri)
        assertArrayEquals(expected, actual)
    }

    @Test
    fun `it should ignore includes that cannot be resolved`() {
        val builder = WorkspaceBuilder()
        val uri = builder.uri("foo.tex")
        val workspace = builder
                .document(uri, "\\include{bar.tex}")
                .workspace

        val expected = arrayOf(uri)
        val actual = relatedDocuments(workspace, uri)
        assertArrayEquals(expected, actual)
    }

    @Test
    fun `it should handle include cycles`() {
        val workspace = WorkspaceBuilder()
                .document("foo.tex", "\\input{bar.tex}")
                .document("bar.tex", "\\input{foo.tex}")
                .workspace

        val expected = workspace.documentsByUri.keys.toTypedArray()
        val actual = relatedDocuments(workspace, workspace.documentsByUri.keys.first())
        assertArrayEquals(expected, actual)
    }
}
