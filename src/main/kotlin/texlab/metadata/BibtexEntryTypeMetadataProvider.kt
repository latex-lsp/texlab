package texlab.metadata

import org.eclipse.lsp4j.MarkupContent
import org.eclipse.lsp4j.MarkupKind

object BibtexEntryTypeMetadataProvider : MetadataProvider {
    override suspend fun getMetadata(name: String): Metadata? {
        val markdown = DOCUMENTATION_BY_NAME[name] ?: return null
        val documentation = MarkupContent().apply {
            kind = MarkupKind.MARKDOWN
            value = markdown.replace(TRIM_REGEX, "")
        }
        return Metadata(name, name, documentation)
    }

    private val TRIM_REGEX = Regex("""^[ \t]+""", RegexOption.MULTILINE)

    private val DOCUMENTATION_BY_NAME = mapOf(
            "article" to
                    """An article in a journal, magazine, newspaper, or other periodical which forms a
                    self-contained unit with its own title. The title of the periodical is given in the
                    journaltitle field. If the issue has its own title in addition to the main title of
                    the periodical, it goes in the issuetitle field. Note that editor and related
                    fields refer to the journal while translator and related fields refer to the article.

                    Required fields: `author`, `title`, `journaltitle`, `year/date`""",

            "book" to
                    """A single-volume book with one or more authors where the authors share credit for
                    the work as a whole. This entry type also covers the function of the `@inbook` type
                    of traditional BibTeX.

                    Required fields: `author`, `title`, `year/date`""",

            "mvbook" to
                    """A multi-volume `@book`. For backwards compatibility, multi-volume books are also
                    supported by the entry type `@book`. However, it is advisable to make use of the
                    dedicated entry type `@mvbook`.

                    Required fields: `author`, `title`, `year/date`""",

            "inbook" to
                    """A part of a book which forms a self-contained unit with its own title. Note that the
                    profile of this entry type is different from standard BibTeX.

                    Required fields: `author`, `title`, `booktitle`, `year/date`""",

            "bookinbook" to
                    """This type is similar to `@inbook` but intended for works originally published as a
                    stand-alone book. A typical example are books reprinted in the collected works of
                    an author.""",

            "suppbook" to
                    """Supplemental material in a `@book`. This type is closely related to the `@inbook`
                    entry type. While `@inbook` is primarily intended for a part of a book with its own
                    title (e. g., a single essay in a collection of essays by the same author), this type is
                    provided for elements such as prefaces, introductions, forewords, afterwords, etc.
                    which often have a generic title only. Style guides may require such items to be
                    formatted differently from other `@inbook` items. The standard styles will treat this
                    entry type as an alias for `@inbook`.""",

            "booklet" to
                    """A book-like work without a formal publisher or sponsoring institution. Use the field
                    howpublished to supply publishing information in free format, if applicable. The
                    field type may be useful as well.

                    Required fields: `author/editor`, `title`, `year/date`""",

            "collection" to
                    """A single-volume collection with multiple, self-contained contributions by distinct
                    authors which have their own title. The work as a whole has no overall author but it
                    will usually have an editor.

                    Required fields: `editor`, `title`, `year/date`""",

            "mvcollection" to
                    """A multi-volume `@collection`. For backwards compatibility, multi-volume collections
                    are also supported by the entry type `@collection`. However, it is advisable
                    to make use of the dedicated entry type `@mvcollection`.
                    Required fields: `editor`, `title`, `year/date`""",

            "incollection" to
                    """A contribution to a collection which forms a self-contained unit with a distinct author
                    and title. The `author` refers to the `title`, the `editor` to the `booktitle`, i. e.,
                    the title of the collection.
                    Required fields: `author`, `title`, `booktitle`, `year/date`""",

            "suppcollection" to
                    """Supplemental material in a `@collection`. This type is similar to `@suppbook` but
                    related to the `@collection` entry type. The standard styles will treat this entry
                    type as an alias for `@incollection`.
                    """,

            "manual" to
                    """Technical or other documentation, not necessarily in printed form. The author or
                    editor is omissible.

                    Required fields: `author/editor`, `title`, `year/date`""",

            "misc" to
                    """A fallback type for entries which do not fit into any other category. Use the field
                    howpublished to supply publishing information in free format, if applicable. The
                    field type may be useful as well. author, editor, and year are omissible.

                    Required fields: `author/editor`, `title`, `year/date`""",

            "online" to
                    """An online resource. `author`, `editor`, and `year` are omissible.
                    This entry type is intended for sources such as web sites which are intrinsically
                    online resources. Note that all entry types support the url field. For example, when
                    adding an article from an online journal, it may be preferable to use the `@article`
                    type and its url field.

                    Required fields: `author/editor`, `title`, `year/date`, `url`""",

            "patent" to
                    """A patent or patent request. The number or record token is given in the number
                    field. Use the type field to specify the type and the location field to indicate the
                    scope of the patent, if different from the scope implied by the type. Note that the
                    location field is treated as a key list with this entry type.

                    Required fields: `author`, `title`, `number`, `year/date`""",

            "periodical" to
                    """An complete issue of a periodical, such as a special issue of a journal. The title of
                    the periodical is given in the title field. If the issue has its own title in addition to
                    the main title of the periodical, it goes in the issuetitle field. The editor is
                    omissible.

                    Required fields: `editor`, `title`, `year/date`""",

            "suppperiodical" to
                    """Supplemental material in a `@periodical`. This type is similar to `@suppbook`
                    but related to the `@periodical` entry type. The role of this entry type may be
                    more obvious if you bear in mind that the `@article` type could also be called
                    `@inperiodical`. This type may be useful when referring to items such as regular
                    columns, obituaries, letters to the editor, etc. which only have a generic title. Style
                    guides may require such items to be formatted differently from articles in the strict
                    sense of the word. The standard styles will treat this entry type as an alias for
                    `@article`.""",

            "proceedings" to
                    """A single-volume conference proceedings. This type is very similar to `@collection`.
                    It supports an optional organization field which holds the sponsoring institution.
                    The editor is omissible.

                    Required fields: `title`, `year/date`""",

            "mvproceedings" to
                    """A multi-volume `@proceedings` entry. For backwards compatibility, multi-volume
                    proceedings are also supported by the entry type `@proceedings`. However, it is
                    advisable to make use of the dedicated entry type `@mvproceedings`

                    Required fields: `title`, `year/date`""",

            "inproceedings" to
                    """An article in a conference proceedings. This type is similar to `@incollection`. It
                    supports an optional `organization` field.

                    Required fields: `author`, `title`, `booktitle`, `year/date`""",

            "reference" to
                    """A single-volume work of reference such as an encyclopedia or a dictionary. This is a
                    more specific variant of the generic `@collection` entry type. The standard styles
                    will treat this entry type as an alias for `@collection`.
                    """,

            "mvreference" to
                    """A multi-volume `@reference` entry. The standard styles will treat this entry type
                    as an alias for `@mvcollection`. For backwards compatibility, multi-volume references
                    are also supported by the entry type `@reference`. However, it is advisable
                    to make use of the dedicated entry type `@mvreference`.""",

            "inreference" to
                    """An article in a work of reference. This is a more specific variant of the generic
                    `@incollection` entry type. The standard styles will treat this entry type as an
                    alias for `@incollection`.""",

            "report" to
                    """A technical report, research report, or white paper published by a university or some
                    other institution. Use the `type` field to specify the type of report. The sponsoring
                    institution goes in the `institution` field.

                    Required fields: `author`, `title`, `type`, `institution`, `year/date`""",

            "set" to
                    """An entry set. This entry type is special.""",

            "thesis" to
                    """A thesis written for an educational institution to satisfy the requirements for a degree.
                    Use the `type` field to specify the type of thesis.

                    Required fields: `author`, `title`, `type`, `institution`, `year/date`""",

            "unpublished" to
                    """A work with an author and a title which has not been formally published, such as
                    a manuscript or the script of a talk. Use the fields `howpublished` and `note` to
                    supply additional information in free format, if applicable.

                    Required fields: `author`, `title`, `year/date`""",

            "xdata" to
                    """This entry type is special. `@xdata` entries hold data which may be inherited by other
                    entries using the `xdata` field. Entries of this type only serve as data containers;
                    they may not be cited or added to the bibliography.""",

            "conference" to
                    """A legacy alias for `@inproceedings`.""",

            "electronic" to
                    """An alias for `@online`.""",

            "mastersthesis" to
                    """Similar to `@thesis` except that the `type` field is optional and defaults to the
                    localised term ‘Master’s thesis’. You may still use the `type` field to override that.""",

            "phdthesis" to
                    """Similar to `@thesis` except that the `type` field is optional and defaults to the
                    localised term ‘PhD thesis’. You may still use the `type` field to override that.""",

            "techreport" to
                    """Similar to `@report` except that the `type` field is optional and defaults to the
                    localised term ‘technical report’. You may still use the `type` field to override that.""",

            "www" to
                    """An alias for `@online`, provided for `jurabib` compatibility.""",

            "artwork" to
                    """Works of the visual arts such as paintings, sculpture, and installations.""",

            "audio" to
                    """Audio recordings, typically on audio cd, dvd, audio cassette, or similar media. See
                    also `@music`.""",

            "bibnote" to
                    """This special entry type is not meant to be used in the `bib` file like other types. It is
                    provided for third-party packages like `notes2bib` which merge notes into the bibliography.
                    The notes should go into the `note` field. Be advised that the `@bibnote`
                    type is not related to the `\defbibnote` command in any way. `\defbibnote`
                    is for adding comments at the beginning or the end of the bibliography, whereas
                    the `@bibnote` type is meant for packages which render endnotes as bibliography
                    entries.""",

            "commentary" to
                    """Commentaries which have a status different from regular books, such as legal commentaries.""",

            "image" to
                    """Images, pictures, photographs, and similar media.""",

            "jurisdiction" to
                    """Court decisions, court recordings, and similar things.""",

            "legislation" to
                    """Laws, bills, legislative proposals, and similar things.""",

            "legal" to
                    """Legal documents such as treaties.""",

            "letter" to
                    """Personal correspondence such as letters, emails, memoranda, etc.""",

            "movie" to
                    """Motion pictures. See also `@video`.""",

            "music" to
                    """Musical recordings. This is a more specific variant of `@audio`.""",

            "performance" to
                    """Musical and theatrical performances as well as other works of the performing arts.
                    This type refers to the event as opposed to a recording, a score, or a printed play.""",

            "review" to
                    """Reviews of some other work. This is a more specific variant of the `@article` type.
                    The standard styles will treat this entry type as an alias for `@article`.""",

            "software" to
                    """Computer software.""",

            "standard" to
                    """National and international standards issued by a standards body such as the International
                    Organization for Standardization.""",

            "video" to
                    """Audiovisual recordings, typically on dvd, vhs cassette, or similar media. See also
                    `@movie`."""
    )
}
