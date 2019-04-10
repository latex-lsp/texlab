package texlab.rename

import io.kotlintest.data.forall
import io.kotlintest.matchers.types.shouldBeNull
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import io.kotlintest.tables.row
import org.eclipse.lsp4j.TextEdit
import texlab.WorkspaceBuilder
import texlab.range

class BibtexEntryRenameProviderTests : StringSpec({
    "it should be able to rename a BibTeX entry and a citation" {
        forall(row(0, 0, 9), row(1, 1, 6)) { index, line, character ->
            WorkspaceBuilder().apply {
                val uri1 = document("foo.bib", "@article{foo, bar = baz}")
                val uri2 = document("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}")
                val uri = listOf(uri1, uri2)[index]
                val edit = rename(BibtexEntryRenameProvider, uri, line, character, "qux")!!
                edit.changes.size.shouldBe(2)
                edit.changes
                        .getValue(uri1.toString())
                        .shouldBe(listOf(TextEdit(range(0, 9, 0, 12), "qux")))

                edit.changes.getValue(uri2.toString())
                        .shouldBe(listOf(TextEdit(range(1, 6, 1, 9), "qux")))
            }
        }
    }

    "it should not rename BibTeX field names" {
        WorkspaceBuilder().apply {
            val uri = document("foo.bib", "@article{foo, bar = baz}")
            val edit = rename(BibtexEntryRenameProvider, uri, 0, 14, "qux")
            edit.shouldBeNull()
        }
    }
})
