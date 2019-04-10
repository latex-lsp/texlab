package texlab.highlight

import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import org.eclipse.lsp4j.DocumentHighlight
import org.eclipse.lsp4j.DocumentHighlightKind
import texlab.WorkspaceBuilder
import texlab.range

class LatexLabelHighlightProviderTests : StringSpec({
    "it should highlight all references of a label" {
        WorkspaceBuilder().apply {
            val highlight1 = DocumentHighlight(range(0, 7, 0, 10), DocumentHighlightKind.Write)
            val highlight2 = DocumentHighlight(range(1, 5, 1, 8), DocumentHighlightKind.Read)
            val uri = document("foo.tex", "\\label{foo}\n\\ref{foo}")
            val highlights = highlight(LatexLabelHighlightProvider, uri, 0, 7)
            highlights.shouldBe(listOf(highlight1, highlight2))
        }
    }

    "it should return an empty list if no label is selected" {
        WorkspaceBuilder().apply {
            val uri = document("foo.tex", "")
            val highlights = highlight(LatexLabelHighlightProvider, uri, 0, 0)
            highlights.shouldBe(emptyList())
        }
    }

    "it should ignore BibTeX documents" {
        WorkspaceBuilder().apply {
            val uri = document("foo.bib", "")
            val highlights = highlight(LatexLabelHighlightProvider, uri, 0, 0)
            highlights.shouldBe(emptyList())
        }
    }
})
