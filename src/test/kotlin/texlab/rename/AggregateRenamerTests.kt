package texlab.rename

import org.eclipse.lsp4j.WorkspaceEdit
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class AggregateRenamerTests {
    private fun createRenamer(edit: WorkspaceEdit?): Renamer {
        return object : Renamer {
            override fun rename(request: RenameRequest): WorkspaceEdit? {
                return edit
            }
        }
    }

    @Test
    fun `it should rename the first result`() {
        val edit1 = WorkspaceEdit().apply { changes = mapOf("foo.tex" to emptyList()) }
        val edit2 = WorkspaceEdit().apply { changes = mapOf("bar.tex" to emptyList()) }
        val renamer1 = createRenamer(null)
        val renamer2 = createRenamer(edit1)
        val renamer3 = createRenamer(edit2)
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .document("bar.tex", "")
                .rename("foo.tex", 0, 0, "qux")

        val aggregateRenamer = AggregateRenamer(renamer1, renamer2, renamer3)
        assertEquals(edit1, aggregateRenamer.rename(request))
    }

    @Test
    fun `it should return null when no results are found`() {
        val renamer1 = createRenamer(null)
        val renamer2 = createRenamer(null)
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .rename("foo.tex", 0, 0, "bar")

        val aggregateRenamer = AggregateRenamer(renamer1, renamer2)
        assertNull(aggregateRenamer.rename(request))
    }
}
