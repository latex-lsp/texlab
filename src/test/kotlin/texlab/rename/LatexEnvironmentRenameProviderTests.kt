package texlab.rename

import io.kotlintest.matchers.types.shouldBeNull
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import org.eclipse.lsp4j.TextEdit
import texlab.WorkspaceBuilder
import texlab.range

class LatexEnvironmentRenameProviderTests : StringSpec({
    "it should rename LaTeX environments" {
        WorkspaceBuilder().apply {
            val uri = document("foo.tex", "\\begin{foo}\n\\end{bar}")
            val edit = rename(LatexEnvironmentRenameProvider, uri, 0, 8, "baz")!!
            edit.changes.size.shouldBe(1)
            edit.changes
                    .getValue(uri.toString())
                    .shouldBe(listOf(
                            TextEdit(range(0, 7, 0, 10), "baz"),
                            TextEdit(range(1, 5, 1, 8), "baz")))
        }
    }

    "it should not rename LaTeX commands" {
        WorkspaceBuilder().apply {
            val uri = document("foo.tex", "\\begin{foo}\n\\end{bar}")
            val edit = rename(LatexEnvironmentRenameProvider, uri, 0, 5, "baz")
            edit.shouldBeNull()
        }
    }

    "it should ignore BibTeX documents" {
        WorkspaceBuilder().apply {
            val uri = document("foo.bib", "")
            val edit = rename(LatexEnvironmentRenameProvider, uri, 0, 0, "baz")
            edit.shouldBeNull()
        }
    }
})
