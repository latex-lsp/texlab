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
        val workspace = WorkspaceBuilder()
                .document("foo.tex", "\\include{bar/baz}")
                .document("bar/baz.tex", "")
                .workspace

        val expected = workspace.documents
                .map { it.uri }
                .toTypedArray()
        val actual = relatedDocuments(workspace, workspace.documents[0].uri)
        assertArrayEquals(expected, actual)
    }

    @Test
    fun `it should ignore invalid includes`() {
        val workspace = WorkspaceBuilder()
                .document("foo.tex", "\\include{<foo>?|bar|:}\n\\include{}")
                .workspace

        val expected = arrayOf(workspace.documents[0].uri)
        val actual = relatedDocuments(workspace, workspace.documents[0].uri)
        assertArrayEquals(expected, actual)
    }

    @Test
    fun `it should find related bibliographies`() {
        val workspace = WorkspaceBuilder()
                .document("foo.tex", "\\addbibresource{bar.bib}")
                .document("bar.bib", "")
                .workspace

        val expected = workspace.documents
                .map { it.uri }
                .toTypedArray()
        val actual = relatedDocuments(workspace, workspace.documents[0].uri)
        assertArrayEquals(expected, actual)
    }

    @Test
    fun `it should ignore includes that cannot be resolved`() {
        val workspace = WorkspaceBuilder()
                .document("foo.tex", "\\include{bar.tex}")
                .workspace

        val expected = arrayOf(workspace.documents[0].uri)
        val actual = relatedDocuments(workspace, workspace.documents[0].uri)
        assertArrayEquals(expected, actual)
    }

    @Test
    fun `it should handle include cycles`() {
        val workspace = WorkspaceBuilder()
                .document("foo.tex", "\\input{bar.tex}")
                .document("bar.tex", "\\input{foo.tex}")
                .workspace

        val expected = workspace.documents
                .map { it.uri }
                .toTypedArray()
        val actual = relatedDocuments(workspace, workspace.documents[0].uri)
        assertArrayEquals(expected, actual)
    }
}
