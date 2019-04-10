package texlab.rename

import io.kotlintest.matchers.types.shouldBeNull
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import org.eclipse.lsp4j.TextEdit
import texlab.WorkspaceBuilder
import texlab.range

class LatexCommandRenameProviderTests : StringSpec({
    "it should rename LaTeX commands in related documents" {
        WorkspaceBuilder().apply {
            val uri1 = document("foo.tex", "\\include{bar.tex}\n\\baz")
            val uri2 = document("bar.tex", "\\baz")
            val edit = rename(LatexCommandRenameProvider, uri1, 1, 2, "qux")!!
            edit.changes.size.shouldBe(2)
            edit.changes
                    .getValue(uri1.toString())
                    .shouldBe(listOf(TextEdit(range(1, 0, 1, 4), "\\qux")))

            edit.changes
                    .getValue(uri2.toString())
                    .shouldBe(listOf(TextEdit(range(0, 0, 0, 4), "\\qux")))
        }
    }

    "it should ignore BibTeX documents" {
        WorkspaceBuilder().apply {
            val uri = document("foo.bib", "\\foo")
            val edit = rename(LatexCommandRenameProvider, uri, 0, 1, "baz")
            edit.shouldBeNull()
        }
    }
})
