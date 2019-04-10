import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import texlab.WorkspaceBuilder

class WorkspaceTests : StringSpec({
    "relatedDocuments should append extensions when analyzing includes" {
        WorkspaceBuilder().apply {
            val uri1 = document("foo.tex", "\\include{bar/baz}")
            val uri2 = document("bar/baz.tex", "")
            workspace.relatedDocuments(uri1)
                    .map { it.uri }
                    .shouldBe(listOf(uri1, uri2))
        }
    }

    "relatedDocuments should ignore invalid includes" {
        WorkspaceBuilder().apply {
            val uri = document("foo.tex", "\\include{<foo>?|bar|:}\n\\include{}")
            workspace.relatedDocuments(uri)
                    .map { it.uri }
                    .shouldBe(listOf(uri))
        }
    }

    "relatedDocuments should find related bibliographies" {
        WorkspaceBuilder().apply {
            val uri1 = document("foo.tex", "\\addbibresource{bar.bib}")
            val uri2 = document("bar.bib", "")
            workspace.relatedDocuments(uri1)
                    .map { it.uri }
                    .shouldBe(listOf(uri1, uri2))
        }
    }

    "relatedDocuments should ignore includes that cannot be resolved" {
        WorkspaceBuilder().apply {
            val uri = document("foo.tex", "\\include{bar.tex}")
            workspace.relatedDocuments(uri)
                    .map { it.uri }
                    .shouldBe(listOf(uri))
        }
    }

    "relatedDocuments should handle include cycles" {
        WorkspaceBuilder().apply {
            val uri1 = document("foo.tex", "\\input{bar.tex}")
            val uri2 = document("bar.tex", "\\input{foo.tex}")
            workspace.relatedDocuments(uri1)
                    .map { it.uri }
                    .shouldBe(listOf(uri1, uri2))
        }
    }

    "findParent should behave as expected" {
        WorkspaceBuilder().apply {
            val uri1 = document("foo.tex", "\\input{bar.tex}")
            val uri2 = document("bar.tex", "\\begin{document}\\end{document}")
            workspace.findParent(uri1).uri.shouldBe(uri2)
        }
    }

    "findParent should return the document if there is no parent" {
        WorkspaceBuilder().apply {
            val uri = document("foo.tex", "")
            document("bar.tex", "\\begin{document}\\end{document}")
            workspace.findParent(uri).uri.shouldBe(uri)
        }
    }
})
