package texlab.completion.bibtex

enum class BibtexField {
    ABSTRACT,
    ADDENDUM,
    AFTERWORD,
    ANNOTATION,
    ANNOTATOR,
    AUTHOR,
    AUTHOR_TYPE,
    BOOK_AUTHOR,
    BOOK_PAGINATION,
    BOOK_SUBTITLE,
    BOOK_TITLE,
    BOOK_TITLE_ADDON,
    CHAPTER,
    COMMENTATOR,
    DATE,
    DOI,
    EDITION,
    EDITOR,
    EDITOR_A,
    EDITOR_B,
    EDITOR_C,
    EDITOR_TYPE,
    EDITOR_A_TYPE,
    EDITOR_B_TYPE,
    EDITOR_C_TYPE,
    EID,
    ENTRY_SUB_TYPE,
    EPRINT,
    EPRINT_CLASS,
    EPRINT_TYPE,
    EVENT_DATE,
    EVENT_TITLE,
    EVENT_TITLE_ADDON,
    FILE,
    FOREWORD,
    HOLDER,
    HOW_PUBLISHED,
    INDEX_TITLE,
    INSTITUTION,
    INTRODUCTION,
    ISAN,
    ISBN,
    ISMN,
    ISRN,
    ISSN,
    ISSUE,
    ISSUE_SUBTITLE,
    ISSUE_TITLE,
    ISWC,
    JOURNAL_SUBTITLE,
    JOURNAL_TITLE,
    LABEL,
    LANGUAGE,
    LIBRARY,
    LOCATION,
    MAIN_SUBTITLE,
    MAIN_TITLE,
    MAIN_TITLE_ADDON,
    MONTH,
    NAME_ADDON,
    NOTE,
    NUMBER,
    ORGANIZATION,
    ORIG_DATE,
    ORIG_LANGUAGE,
    ORIG_LOCATION,
    ORIG_PUBLISHER,
    ORIG_TITLE,
    PAGES,
    PAGE_TOTAL,
    PAGINATION,
    PART,
    PUBLISHER,
    PUB_STATE,
    REPRINT_TITLE,
    SERIES,
    SHORT_AUTHOR,
    SHORT_EDITOR,
    SHORTHAND,
    SHORTHAND_INTRO,
    SHORT_JOURNAL,
    SHORT_SERIES,
    SHORT_TITLE,
    SUBTITLE,
    TITLE,
    TITLE_ADDON,
    TRANSLATOR,
    TYPE,
    URL,
    URL_DATE,
    VENUE,
    VERSION,
    VOLUME,
    VOLUMES,
    YEAR,
    CROSSREF,
    ENTRY_SET,
    EXECUTE,
    GENDER,
    LANG_ID,
    LANG_ID_OPTS,
    IDS,
    INDEX_SORT_TITLE,
    KEYWORDS,
    OPTIONS,
    PRESORT,
    RELATED,
    RELATED_OPTIONS,
    RELATED_TYPE,
    RELATED_STRING,
    SORT_KEY,
    SORT_NAME,
    SORT_SHORTHAND,
    SORT_TITLE,
    SORT_YEAR,
    XDATA,
    XREF,
    NAME_A,
    NAME_B,
    NAME_C,
    NAME_A_TYPE,
    NAME_B_TYPE,
    NAME_C_TYPE,
    LIST_A,
    LIST_B,
    LIST_C,
    LIST_D,
    LIST_E,
    LIST_F,
    USER_A,
    USER_B,
    USER_C,
    USER_D,
    USER_E,
    USER_F,
    VERB_A,
    VERB_B,
    VERB_C,
    ADDRESS,
    ANNOTE,
    ARCHIVE_PREFIX,
    JOURNAL,
    KEY,
    PDF,
    PRIMARY_CLASS,
    SCHOOL;

    override fun toString(): String = super.toString().replace("_", "").toLowerCase()

    fun documentation(): String {
        return when (this) {
            ABSTRACT ->
                """This field is intended for recording abstracts in a bib file,
                    to be printed by a special bibliography style.
                    It is not used by all standard bibliography styles."""
            ADDENDUM ->
                """Miscellaneous bibliographic data to be printed at the end of the entry.
                    This is similar to the `note` field except that it is printed at the
                    end of the bibliography entry."""
            AFTERWORD ->
                """The author(s) of an afterword to the work. If the author of the afterword is identical to
                    the `editor` and/or `translator`, the standard styles will automatically concatenate these
                    fields in the bibliography. See also `introduction` and `foreword`."""
            ANNOTATION ->
                """This field may be useful when implementing a style for annotated bibliographies.
                    It is not used by all standard bibliography styles. Note that this field is completely
                    unrelated to `annotator`. The `annotator` is the author of annotations which are
                    part of the work cited."""
            ANNOTATOR ->
                """The author(s) of annotations to the work. If the annotator is identical to the `editor`
                    and/or `translator`, the standard styles will automatically concatenate these fields
                    in the bibliography. See also `commentator`."""
            AUTHOR ->
                """The author(s) of the `title`."""
            AUTHOR_TYPE ->
                """The type of author. This field will affect the string (if any) used to
                    introduce the author. Not used by the standard bibliography styles."""
            BOOK_AUTHOR ->
                """The author(s) of the `booktitle`."""
            BOOK_PAGINATION ->
                """If the work is published as part of another one, this is the pagination scheme of the
                    enclosing work, i. e., `bookpagination` relates to `pagination` like `booktitle` to `title`.
                    The value of this field will affect the formatting of the `pages` and `pagetotal` fields.
                    The key should be given in the singular form. Possible
                    keys are `page`, `column`, `line`, `verse`, `section`, and `paragraph`. See also `pagination`."""
            BOOK_SUBTITLE ->
                """The subtitle related to the `booktitle`. If the subtitle field refers to a work
                    which is part of a larger publication, a possible subtitle of the main work is given in
                    this field. See also `subtitle`."""
            BOOK_TITLE ->
                """If the `title` field indicates the title of a work which is part of a larger publication,
                    the title of the main work is given in this field. See also `title`."""
            BOOK_TITLE_ADDON ->
                """An annex to the `booktitle`, to be printed in a different font."""
            CHAPTER ->
                """A chapter or section or any other unit of a work."""
            COMMENTATOR ->
                """The author(s) of a commentary to the work. Note that this field is intended for
                    commented editions which have a commentator in addition to the author. If the
                    work is a stand-alone commentary, the commentator should be given in the `author` field.
                    If the commentator is identical to the `editor` and/or `translator`, the
                    standard styles will automatically concatenate these fields in the bibliography.
                    See also `annotator`."""
            DATE ->
                """The publication date. See also `month` and `year`."""
            DOI ->
                """The Digital Object Identifier of the work."""
            EDITION ->
                """The edition of a printed publication. This must be an integer, not an ordinal.
                    Don’t say `edition={First}` or `edition={1st}` but `edition={1}`.
                    The bibliography style converts this to a language dependent ordinal.
                    It is also possible to give the edition as a literal string,
                    for example "Third, revised and expanded edition"."""
            EDITOR ->
                """The editor(s) of the `title`, `booktitle`, or `maintitle`, depending on the entry type.
                    Use the `editortype` field to specify the role if it is different from `editor`."""
            EDITOR_A ->
                """A secondary editor performing a different editorial role,
                    such as compiling, redacting, etc. Use the `editoratype` field to specify the role."""
            EDITOR_B ->
                """Another secondary editor performing a different role.
                    Use the `editorbtype` field to specify the role."""
            EDITOR_C ->
                """Another secondary editor performing a different role.
                    Use the `editorctype` field to specify the role."""
            EDITOR_TYPE ->
                """The type of editorial role performed by the `editor`.
                    Roles supported by default are `editor`, `compiler`, `founder`, `continuator`,
                    `redactor`, `reviser`, `collaborator`, `organizer`.
                    The role `editor` is the default. In this case, the field is omissible."""
            EDITOR_A_TYPE ->
                """Similar to `editortype` but referring to the `editora` field."""
            EDITOR_B_TYPE ->
                """Similar to `editortype` but referring to the `editorb` field."""
            EDITOR_C_TYPE ->
                """Similar to `editortype` but referring to the `editorc` field."""
            EID ->
                """The electronic identifier of an `@article`."""
            ENTRY_SUB_TYPE ->
                """This field, which is not used by the standard styles,
                    may be used to specify a subtype of an entry type.
                    This may be useful for bibliography styles which support a
                    finergrained set of entry types."""
            EPRINT ->
                """The electronic identifier of an online publication.
                    This is roughly comparable to a doi but specific to a certain archive,
                    repository, service, or system. See also `eprinttype` and `eprintclass`."""
            EPRINT_CLASS ->
                """Additional information related to the resource indicated by the `eprinttype` field.
                    This could be a section of an archive, a path indicating a service, a classification of
                    some sort, etc. See also`eprint` and `eprinttype`."""
            EPRINT_TYPE ->
                """The type of `eprint` identifier, e. g., the name of the archive, repository, service, or
                    system the `eprint` field refers to. See also `eprint` and `eprintclass`."""
            EVENT_DATE ->
                """The date of a conference, a symposium, or some other event in `@proceedings`
                    and `@inproceedings` entries. See also `eventtitle` and `venue`."""
            EVENT_TITLE ->
                """The title of a conference, a symposium, or some other event in `@proceedings` and
                    `@inproceedings` entries. Note that this field holds the plain title of the event.
                    Things like "Proceedings of the Fifth XYZ Conference" go into the
                    `titleaddon` or `booktitleaddon` field, respectively. See also `eventdate` and `venue`."""
            EVENT_TITLE_ADDON ->
                """An annex to the `eventtitle` field. Can be used for known event acronyms, for example."""
            FILE ->
                """A local link to a PDF or other version of the work. Not used by the standard bibliography styles."""
            FOREWORD ->
                """The author(s) of a foreword to the work. If the author of the foreword is
                    identical to the `editor` and/or `translator`, the standard styles will automatically
                    concatenate these fields in the bibliography. See also `introduction` and `afterword`."""
            HOLDER ->
                """The holder(s) of a `@patent`, if different from the `author`.
                    Note that corporate holders need to be wrapped in an additional set of braces."""
            HOW_PUBLISHED ->
                """A publication notice for unusual publications which do not fit
                    into any of the common categories."""
            INDEX_TITLE ->
                """A title to use for indexing instead of the regular `title` field.
                    This field may be useful if you have an entry with a title like
                    "An Introduction to …" and want that indexed
                    as "Introduction to …, An". Style authors should note that `biblatex` automatically
                    copies the value of the `title` field to `indextitle` if the latter field is undefined."""
            INSTITUTION ->
                """The name of a university or some other institution, depending on the entry type.
                    Traditional BibTeX uses the field name `school` for theses, which is supported as an alias."""
            INTRODUCTION ->
                """The author(s) of an introduction to the work. If the author of the introduction is
                    identical to the `editor` and/or `translator`, the standard styles will automatically
                    concatenate these fields in the bibliography. See also `foreword` and `afterword`."""
            ISAN ->
                """The International Standard Audiovisual Number of an audiovisual work.
                    Not used by the standard bibliography styles."""
            ISBN ->
                """The International Standard Book Number of a book."""
            ISMN ->
                """The International Standard Music Number for printed music such
                    as musical scores. Not used by the standard bibliography styles."""
            ISRN ->
                """The International Standard Technical Report Number of a technical report."""
            ISSN ->
                """The International Standard Serial Number of a periodical."""
            ISSUE ->
                """The issue of a journal. This field is intended for journals whose
                    individual issues are identified by a designation such as ‘Spring’ or ‘Summer’
                    rather than the month or a number. The placement of `issue` is similar to `month` and `number`,
                    integer ranges and short designators are better written to the number field.
                    See also `month` and `number`."""
            ISSUE_SUBTITLE ->
                """The subtitle of a specific issue of a journal or other periodical."""
            ISSUE_TITLE ->
                """The title of a specific issue of a journal or other periodical."""
            ISWC ->
                """The International Standard Work Code of a musical work.
                    Not used by the standard bibliography styles."""
            JOURNAL_SUBTITLE ->
                """The subtitle of a journal, a newspaper, or some other periodical."""
            JOURNAL_TITLE ->
                """The name of a journal, a newspaper, or some other periodical."""
            LABEL ->
                """A designation to be used by the citation style as a substitute for the regular label
                    if any data required to generate the regular label is missing.
                    For example, when an author-year citation style is generating a citation for an entry
                    which is missing the author or the year, it may fall back to `label`.
                    Note that, in contrast to `shorthand`, `label` is only used as a fallback.
                    See also `shorthand`."""
            LANGUAGE ->
                """The language(s) of the work. Languages may be specified literally or as localisation keys.
                    If localisation keys are used, the prefix lang is omissible. See also `origlanguage`."""
            LIBRARY ->
                """This field may be useful to record information such as a library name and a call number.
                    This may be printed by a special bibliography style if desired.
                    Not used by the standard bibliography styles."""
            LOCATION ->
                """The place(s) of publication, i. e., the location of the `publisher` or `institution`,
                    depending on the entry type. Traditional BibTeX uses the field name `address`,
                    which is supported as an alias.
                    With `@patent` entries, this list indicates the scope of a patent."""
            MAIN_SUBTITLE ->
                """The subtitle related to the `maintitle`. See also `subtitle`."""
            MAIN_TITLE ->
                """The main title of a multi-volume book, such as *Collected Works*.
                    If the `title` or `booktitle` field indicates the title of a single volume
                    which is part of multi-volume book, the title of the complete work is given in this field."""
            MAIN_TITLE_ADDON ->
                """An annex to the `maintitle`, to be printed in a different font."""
            MONTH ->
                """The publication month. This must be an integer, not an ordinal or a string.
                    Don’t say `month={January}` but `month={1}`. The bibliography style converts
                    this to a language dependent string or ordinal where required.
                    This field is a literal field only when given explicitly in the
                    data (for plain BibTeX compatibility for example).
                    It is however better to use the `date` field as this supports many more features."""
            NAME_ADDON ->
                """An addon to be printed immediately after the author name in the bibliography.
                    Not used by the standard bibliography styles. This field may be useful to add an alias
                    or pen name (or give the real name if the pseudonym is commonly used to refer to that author)."""
            NOTE ->
                """Miscellaneous bibliographic data which does not fit into any other field.
                    The note field may be used to record bibliographic data in a free format.
                    Publication facts such as "Reprint of the edition London 1831" are typical
                    candidates for the note field. See also `addendum`."""
            NUMBER ->
                """The number of a journal or the volume/number of a book in a `series`.
                    See also `issue`. With `@patent` entries, this is the number
                    or record token of a patent or patent request.
                    Normally this field will be an integer or an integer range,
                    but in certain cases it may also contain "S1", "Suppl. 1",
                    in these cases the output should be scrutinised carefully."""
            ORGANIZATION ->
                """The organization(s) that published a `@manual` or an `@online` resource,
                    or sponsored a conference."""
            ORIG_DATE ->
                """If the work is a translation, a reprint, or something similar,
                    the publication date of the original edition.
                    Not used by the standard bibliography styles. See also `date`."""
            ORIG_LANGUAGE ->
                """If the work is a translation, the language(s) of the original work. See also `language`."""
            ORIG_LOCATION ->
                """If the work is a translation, a reprint, or something similar,
                    the location of the original edition. Not used by the standard bibliography styles.
                    See also `location`."""
            ORIG_PUBLISHER ->
                """If the work is a translation, a reprint, or something similar,
                    the publisher of the original edition. Not used by the standard bibliography styles.
                    See also `publisher`."""
            ORIG_TITLE ->
                """If the work is a translation, the `title` of the original work.
                    Not used by the standard bibliography styles. See also `title`."""
            PAGES ->
                """One or more page numbers or page ranges.
                    If the work is published as part of another one,
                    such as an article in a journal or a collection, this field holds the relevant page
                    range in that other work. It may also be used to limit the reference to a specific part
                    of a work (a chapter in a book, for example)."""
            PAGE_TOTAL ->
                """The total number of pages of the work."""
            PAGINATION ->
                """The pagination of the work. The value of this field will affect the
                    formatting the *postnote* argument to a citation command.
                    The key should be given in the singular form.
                    Possible keys are `page`, `column`, `line`, `verse`, `section`, and `paragraph`.
                    See also `bookpagination`."""
            PART ->
                """The number of a partial volume. This field applies to books only, not to journals.
                    It may be used when a logical volume consists of two or more physical ones.
                    In this case the number of the logical volume goes in the `volume` field and
                    the number of the part of that volume in the `part` field. See also `volume`."""
            PUBLISHER ->
                """The name(s) of the publisher(s)."""
            PUB_STATE ->
                """The publication state of the work, e. g., 'in press'."""
            REPRINT_TITLE ->
                """The title of a reprint of the work. Not used by the standard styles."""
            SERIES ->
                """The name of a publication series, such as "Studies in …",
                    or the number of a journal series. Books in a publication series are usually numbered.
                    The number or volume of a book in a series is given in the `number` field.
                    Note that the `@article` entry type makes use of the `series` field as well,
                    but handles it in a special way."""
            SHORT_AUTHOR ->
                """The author(s) of the work, given in an abbreviated form.
                    This field is mainly intended for abbreviated forms of corporate authors."""
            SHORT_EDITOR ->
                """The editor(s) of the work, given in an abbreviated form.
                    This field is mainly intended for abbreviated forms of corporate editors."""
            SHORTHAND ->
                """A special designation to be used by the citation style instead of the usual label.
                    If defined, it overrides the default label. See also `label`."""
            SHORTHAND_INTRO ->
                """The verbose citation styles which comes with this package use a phrase like
                    "henceforth cited as [shorthand]" to introduce shorthands on the first citation.
                    If the `shorthandintro` field is defined, it overrides the standard phrase.
                    Note that the alternative phrase must include the shorthand."""
            SHORT_JOURNAL ->
                """A short version or an acronym of the `journaltitle`.
                    Not used by the standard bibliography styles."""
            SHORT_SERIES ->
                """A short version or an acronym of the `series` field.
                    Not used by the standard bibliography styles."""
            SHORT_TITLE ->
                """The title in an abridged form. This field is usually not included in the bibliography.
                    It is intended for citations in author-title format.
                    If present, the author-title citation styles use this field instead of `title`."""
            SUBTITLE ->
                """The subtitle of the work."""
            TITLE ->
                """The title of the work."""
            TITLE_ADDON ->
                """An annex to the `title`, to be printed in a different font."""
            TRANSLATOR ->
                """The translator(s) of the `title` or `booktitle`, depending on the entry type.
                    If the translator is identical to the `editor`,
                    the standard styles will automatically concatenate these fields in the bibliography."""
            TYPE ->
                """The type of a `manual`, `patent`, `report`, or `thesis`."""
            URL ->
                """The URL of an online publication. If it is not URL-escaped (no ‘%’ chars)
                    it will be URI-escaped according to RFC 3987, that is,
                    even Unicode chars will be correctly escaped."""
            URL_DATE ->
                """The access date of the address specified in the `url` field."""
            VENUE ->
                """The location of a conference, a symposium, or some other event in `@proceedings`
                    and `@inproceedings` entries. Note that the `location` list holds the place of publication.
                    It therefore corresponds to the `publisher` and `institution` lists.
                    The location of the event is given in the `venue` field. See also `eventdate` and `eventtitle`."""
            VERSION ->
                """The revision number of a piece of software, a manual, etc."""
            VOLUME ->
                """The volume of a multi-volume book or a periodical.
                    It is expected to be an integer, not necessarily in arabic numerals
                    since `biber` will automatically from roman numerals or arabic letter to integers internally
                    for sorting purposes. See also `part`.
                    See the `noroman` option which can be used to suppress roman numeral parsing.
                    This can help in cases where there is an ambiguity between parsing as
                    roman numerals or alphanumeric (e.g. ‘C’)."""
            VOLUMES ->
                """The total number of volumes of a multi-volume work.
                    Depending on the entry type, this field refers to `title` or `maintitle`.
                    It is expected to be an integer, not necessarily in arabic numerals since `biber`
                    will automatically from roman numerals or arabic letter to integers internally
                    for sorting purposes. See the `noroman` option which can be used to suppress roman numeral parsing.
                    This can help in cases where there is an ambiguity between parsing as
                    roman numerals or alphanumeric (e.g. ‘C’)."""
            YEAR ->
                """The year of publication. This field is a literal field only when given explicitly
                    in the data (for plain BibTeX compatibility for example).
                    It is however better to use the `date` field as this is compatible with plain years too
                    and supports many more features."""
            CROSSREF ->
                """This field holds an entry key for the cross-referencing feature.
                    Child entries with a `crossref` field inherit data from the parent entry specified
                    in the `crossref` field. If the number of child entries referencing a specific
                    parent entry hits a certain threshold, the parent entry is automatically added
                    to the bibliography even if it has not been cited explicitly. The threshold
                    is settable with the `mincrossrefs` package option.
                    Style authors should note that whether or not the `crossref` fields of the child
                    entries are defined on the `biblatex` level depends on the availability of the parent entry.
                    If the parent entry is available, the `crossref`
                    fields of the child entries will be defined. If not, the child entries still inherit the
                    data from the parent entry but their `crossref` fields will be undefined. Whether
                    the parent entry is added to the bibliography implicitly because of the threshold or
                    explicitly because it has been cited does not matter. See also the `xref` field."""
            ENTRY_SET ->
                """This field is specific to entry sets.
                    This field is consumed by the backend processing and does not appear in the `.bbl`."""
            EXECUTE ->
                """A special field which holds arbitrary TeX code to be executed whenever the data of the
                    respective entry is accessed. This may be useful to handle special cases.
                    Conceptually, this field is comparable to the hooks `\AtEveryBibitem`, `\AtEveryLositem`,
                    and `\AtEveryCitekey`, except that it is definable on a per-entry
                    basis in the `bib` file. Any code in this field is executed automatically immediately
                    after these hooks."""
            GENDER ->
                """The gender of the author or the gender of the editor, if there is no author.
                    The following identifiers are supported: `sf` (feminine singular, a single female name),
                    `sm` (masculine singular, a single male name), `sn` (neuter singular, a single neuter name),
                    `pf` (feminine plural, a list of female names), `pm` (masculine plural, a list of male names),
                    `pn` (neuter plural, a list of neuter names),`pp` (plural, a mixed gender list of names).
                    This information is only required by special bibliography and citation styles
                    and only in certain languages. For example, a citation style may replace recurrent
                    author names with a term such as 'idem'. If the Latin word is used, as is custom in
                    English and French, there is no need to specify the gender. In German publications,
                    however, such key terms are usually given in German and in this case they are
                    gender-sensitive."""
            LANG_ID ->
                """The language id of the bibliography entry.
                    The alias `hyphenation` is provided for backwards compatibility.
                    The identifier must be a language name known to the `babel/polyglossia` packages.
                    This information may be used to switch hyphenation patterns and localise
                    strings in the bibliography. Note that the language names are case sensitive.
                    The languages currently supported by this package are given in table 2.
                    Note that `babel` treats the identifier `english` as an alias for `british`
                    or `american`, depending on the `babel` version. The `biblatex` package always
                    treats it as an alias for `american`. It is preferable to use the language identifiers
                    `american` and `british` (`babel`) or a language specific option to specify a language variant
                    (`polyglossia`, using the `langidopts` field) to avoid any possible confusion."""
            LANG_ID_OPTS ->
                """For `polyglossia` users, allows per-entry language specific options.
                    The literal value of this field is passed to `polyglossia`’s language switching
                    facility when using the package option `autolang=langname`."""
            IDS ->
                """Citation key aliases for the main citation key.
                    An entry may be cited by any of its aliases and `biblatex` will treat the citation as
                    if it had used the primary citation key. This is to aid users who change their citation
                    keys but have legacy documents which use older keys for the same entry.
                    This field is consumed by the backend processing and does not appear in the `.bbl`."""
            INDEX_SORT_TITLE ->
                """The title used when sorting the index. In contrast to indextitle,
                    this field is used for sorting only. The printed title in the index is the
                    indextitle or the title field. This field may be useful if the title contains
                    special characters or commands which interfere with the sorting of the index.
                    Style authors should note that biblatex automatically copies the value of either the indextitle
                    or the title field to indexsorttitle if the latter field is undefined."""
            KEYWORDS ->
                """A separated list of keywords. These keywords are intended for the bibliography filters,
                    they are usually not printed. Note that with the default separator (comma),
                    spaces around the separator are ignored."""
            OPTIONS ->
                """A separated list of entry options in *key*=*value* notation.
                    This field is used to set options on a per-entry basis.
                    Note that citation and bibliography styles may define additional entry options."""
            PRESORT ->
                """A special field used to modify the sorting order of the bibliography.
                    This field is the first item the sorting routine considers when sorting the bibliography,
                    hence it may be used to arrange the entries in groups.
                    This may be useful when creating subdivided bibliographies with the bibliography filters.
                    This field is consumed by the backend processing and does not appear in the `.bbl`."""
            RELATED ->
                """Citation keys of other entries which have a relationship to this entry.
                    The relationship is specified by the `relatedtype` field."""
            RELATED_OPTIONS ->
                """Per-type options to set for a related entry.
                    Note that this does not set the options on the related entry itself,
                    only the `dataonly` clone which is used as a datasource for the parent entry."""
            RELATED_TYPE ->
                """An identifier which specified the type of relationship for the keys listed in the
                    `related` field. The identifier is a localised bibliography string printed before the
                    data from the related entry list. It is also used to identify type-specific formatting
                    directives and bibliography macros for the related entries."""
            RELATED_STRING ->
                """A field used to override the bibliography string specified by `relatedtype`."""
            SORT_KEY ->
                """A field used to modify the sorting order of the bibliography. Think of this field as
                    the master sort key. If present, `biblatex` uses this field during sorting and ignores
                    everything else, except for the presort field.
                    This field is consumed by the backend processing and does not appear in the `.bbl`."""
            SORT_NAME ->
                """A name or a list of names used to modify the sorting order of the bibliography.
                    If present, this list is used instead of `author` or `editor` when sorting the bibliography.
                    This field is consumed by the backend processing and does not appear in the `.bbl`."""
            SORT_SHORTHAND ->
                """Similar to sortkey but used in the list of shorthands.
                    If present, biblatex uses this field instead of shorthand when sorting the list of shorthands.
                    This is useful if the shorthand field holds shorthands with formatting commands such as `\emph`
                    or `\textbf`. This field is consumed by the backend
                    processing and does not appear in the `.bbl`."""
            SORT_TITLE ->
                """A field used to modify the sorting order of the bibliography.
                    If present, this field is used instead of the title field when sorting the bibliography.
                    The sorttitle field may come in handy if you have an entry with a title like
                    "An Introduction to…" and want that alphabetized under ‘I’ rather than ‘A’.
                    In this case, you could put "Introduction to…" in the sorttitle field.
                    This field is consumed by the backend processing and does not appear in the `.bbl`."""
            SORT_YEAR ->
                """A field used to modify the sorting order of the bibliography.
                    In the default sorting templates, if this field is present,
                    it is used instead of the year field when sorting the bibliography.
                    This field is consumed by the backend processing and does not appear in the `.bbl`."""
            XDATA ->
                """This field inherits data from one or more `@xdata` entries.
                    Conceptually, the `xdata` field is related to crossref and xref:
                    `crossref` establishes a logical parent/child relation and inherits data;
                    `xref` establishes as logical parent/child relation without inheriting data;
                    `xdata` inherits data without establishing a relation.
                    The value of the `xdata` may be a single entry key or a separated list of keys.
                    This field is consumed by the backend processing and does not appear in the `.bbl`."""
            XREF ->
                """This field is an alternative cross-referencing mechanism.
                    It differs from `crossref` in that the child entry will not inherit any data from the
                    parent entry specified in the `xref` field. If the number of child entries
                    referencing a specific parent entry hits a certain threshold,
                    the parent entry is automatically added to the bibliography
                    even if it has not been cited explicitly. The threshold is settable with the `minxrefs`
                    package option. Style authors should note that whether or not the
                    `xref` fields of the child entries are defined on the `biblatex` level depends on the
                    availability of the parent entry. If the parent entry is available, the `xref` fields of
                    the child entries will be defined. If not, their `xref` fields will be undefined. Whether
                    the parent entry is added to the bibliography implicitly because of the threshold or
                    explicitly because it has been cited does not matter. See also the `crossref` field."""
            NAME_A ->
                """Custom lists for special bibliography styles. Not used by the standard bibliography styles."""
            NAME_B ->
                """Custom lists for special bibliography styles. Not used by the standard bibliography styles."""
            NAME_C ->
                """Custom lists for special bibliography styles. Not used by the standard bibliography styles."""
            NAME_A_TYPE ->
                """Similar to `authortype` and `editortype` but referring to the fields `name[a--c]`.
                    Not used by the standard bibliography styles."""
            NAME_B_TYPE ->
                """Similar to `authortype` and `editortype` but referring to the fields `name[a--c]`.
                    Not used by the standard bibliography styles."""
            NAME_C_TYPE ->
                """Similar to `authortype` and `editortype` but referring to the fields `name[a--c]`.
                    Not used by the standard bibliography styles."""
            LIST_A ->
                """Custom lists for special bibliography styles. Not used by the standard bibliography styles."""
            LIST_B ->
                """Custom lists for special bibliography styles. Not used by the standard bibliography styles."""
            LIST_C ->
                """Custom lists for special bibliography styles. Not used by the standard bibliography styles."""
            LIST_D ->
                """Custom lists for special bibliography styles. Not used by the standard bibliography styles."""
            LIST_E ->
                """Custom lists for special bibliography styles. Not used by the standard bibliography styles."""
            LIST_F ->
                """Custom lists for special bibliography styles. Not used by the standard bibliography styles."""
            USER_A ->
                """Custom fields for special bibliography styles. Not used by the standard bibliography styles."""
            USER_B ->
                """Custom fields for special bibliography styles. Not used by the standard bibliography styles."""
            USER_C ->
                """Custom fields for special bibliography styles. Not used by the standard bibliography styles."""
            USER_D ->
                """Custom fields for special bibliography styles. Not used by the standard bibliography styles."""
            USER_E ->
                """Custom fields for special bibliography styles. Not used by the standard bibliography styles."""
            USER_F ->
                """Custom fields for special bibliography styles. Not used by the standard bibliography styles."""
            VERB_A ->
                """Similar to the custom fields except that these are verbatim fields.
                    Not used by the standard bibliography styles."""
            VERB_B ->
                """Similar to the custom fields except that these are verbatim fields.
                    Not used by the standard bibliography styles."""
            VERB_C ->
                """Similar to the custom fields except that these are verbatim fields.
                    Not used by the standard bibliography styles."""
            ADDRESS ->
                """An alias for `location`, provided for BibTeX compatibility.
                    Traditional BibTeX uses the slightly misleading field name `address` for the place of publication,
                    i. e., the location of the publisher, while `biblatex` uses the generic field name `location`."""
            ANNOTE ->
                """An alias for `annotation`, provided for jurabib compatibility."""
            ARCHIVE_PREFIX ->
                """An alias for `eprinttype`, provided for arXiv compatibility."""
            JOURNAL ->
                """An alias for `journaltitle`, provided for BibTeX compatibility."""
            KEY ->
                """An alias for `sortkey`, provided for BibTeX compatibility."""
            PDF ->
                """An alias for `file`, provided for JabRef compatibility."""
            PRIMARY_CLASS ->
                """An alias for `eprintclass`, provided for arXiv compatibility."""
            SCHOOL ->
                """An alias for `institution`, provided for BibTeX compatibility.
                    The `institution` field is used by traditional BibTeX for technical reports
                    whereas the `school` field holds the institution associated with theses.
                    The `biblatex` package employs the generic field name `institution` in both cases."""
        }
    }
}
