package texlab.rename

import io.kotlintest.data.forall
import io.kotlintest.matchers.types.shouldBeNull
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import io.kotlintest.tables.row
import org.eclipse.lsp4j.TextEdit
import texlab.WorkspaceBuilder
import texlab.range

class LatexLabelRenameProviderTests : StringSpec({
    "it should be able to rename LaTeX labels" {
        forall(row(0, 0, 7), row(1, 0, 5)) { index, line, character ->
            WorkspaceBuilder().apply {
                val uri1 = document("foo.tex", "\\label{foo}\n\\include{bar}")
                val uri2 = document("bar.tex", "\\ref{foo}")
                val uri = listOf(uri1, uri2)[index]
                val edit = rename(LatexLabelRenameProvider, uri, line, character, "bar")!!
                edit.changes.size.shouldBe(2)

                edit.changes
                        .getValue(uri1.toString())
                        .shouldBe(listOf(TextEdit(range(0, 7, 0, 10), "bar")))

                edit.changes
                        .getValue(uri2.toString())
                        .shouldBe(listOf(TextEdit(range(0, 5, 0, 8), "bar")))
            }
        }
    }

    "it should not rename LaTeX command arguments" {
        WorkspaceBuilder().apply {
            val uri = document("foo.tex", "\\foo{bar}")
            val edit = rename(LatexLabelRenameProvider, uri, 0, 5, "baz")
            edit.shouldBeNull()
        }
    }

    "it should ignore BibTeX documents" {
        WorkspaceBuilder().apply {
            val uri = document("foo.bib", "")
            val edit = rename(LatexLabelRenameProvider, uri, 0, 0, "bar")
            edit.shouldBeNull()
        }
    }
})
