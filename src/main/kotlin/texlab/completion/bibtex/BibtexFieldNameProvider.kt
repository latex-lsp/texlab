package texlab.completion.bibtex

import org.eclipse.lsp4j.CompletionItem
import texlab.BibtexDocument
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionProvider
import texlab.completion.CompletionRequest
import texlab.contains
import texlab.syntax.bibtex.BibtexEntrySyntax
import texlab.syntax.bibtex.BibtexFieldSyntax

object BibtexFieldNameProvider : CompletionProvider {

    private val FIELD_NAMES = arrayOf("abstract", "addendum", "afterword", "annotation", "annotator", "author",
            "authortype", "bookauthor", "bookpagination", "booksubtitle", "booktitle", "booktitleaddon", "chapter",
            "commentator", "date", "doi", "edition", "editor", "editora", "editorb", "editorc", "editortype",
            "editoratype", "editorbtype", "editorctype", "eid", "entrysubtype", "eprint", "eprintclass", "eprinttype",
            "eventdate", "eventtitle", "eventtitleaddon", "file", "foreword", "holder", "howpublished", "indextitle",
            "institution", "introduction", "isan", "isbn", "ismn", "isrn", "issn", "issue", "issuesubtitle",
            "issuetitle", "iswc", "journalsubtitle", "journaltitle", "label", "language", "library", "location",
            "mainsubtitle", "maintitle", "maintitleaddon", "month", "nameaddon", "note", "number", "organization",
            "origdate", "origlanguage", "origlocation", "origpublisher", "origtitle", "pages", "pagetotal",
            "pagination", "part", "publisher", "pubstate", "reprinttitle", "series", "shortauthor", "shorteditor",
            "shorthand", "shorthandintro", "shortjournal", "shortseries", "shorttitle", "subtitle", "title",
            "titleaddon", "translator", "type", "url", "urldate", "venue", "version", "volume", "volumes", "year",
            "crossref", "entryset", "execute", "gender", "langid", "langidopts", "ids", "indexsorttitle", "keywords",
            "options", "presort", "related", "relatedoptions", "relatedtype", "relatedstring", "sortkey", "sortname",
            "sortshorthand", "sorttitle", "sortyear", "xdata", "xref", "address", "annote", "archiveprefix", "journal",
            "key", "pdf", "primaryclass", "school")

    private val items = FIELD_NAMES.map { CompletionItemFactory.createFieldName(it) }

    override fun complete(request: CompletionRequest): List<CompletionItem> {
        if (request.document !is BibtexDocument) {
            return emptyList()
        }

        val node = request.document.tree.root
                .descendants()
                .lastOrNull { it.range.contains(request.position) }

        val field = node is BibtexFieldSyntax && node.name.range.contains(request.position)
        val entry = node is BibtexEntrySyntax && !node.type.range.contains(request.position)
                && node.name?.range?.contains(request.position) != true
        return if (field || entry) {
            items
        } else {
            emptyList()
        }
    }
}
