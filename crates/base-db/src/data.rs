#[derive(Debug, Clone, Copy)]
pub struct BibtexEntryType<'a> {
   pub name: &'a str,
   pub category: BibtexEntryTypeCategory,
   pub documentation: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum BibtexEntryTypeCategory {
    Misc,
    String,
    Article,
    Thesis,
    Book,
    Part,
    Collection,
}

#[derive(Debug, Clone, Copy)]
pub struct BibtexFieldType<'a> {
   pub name: &'a str,
   pub documentation: &'a str,
}

impl<'a> BibtexEntryType<'a> {
    pub fn find(name: &str) -> Option<Self> {
        BIBTEX_ENTRY_TYPES.iter().find(|ty| ty.name.eq_ignore_ascii_case(name)).copied()
    }    
}

impl<'a> BibtexFieldType<'a> {
    pub fn find(name: &str) -> Option<Self> {
        BIBTEX_FIELD_TYPES.iter().find(|ty| ty.name.eq_ignore_ascii_case(name)).copied()
    }    
}


pub static BIBTEX_ENTRY_TYPES: &[BibtexEntryType<'static>] = &[
    BibtexEntryType {
        name: "preamble",
        category: BibtexEntryTypeCategory::Misc,
        documentation: None,
    },
    BibtexEntryType {
        name: "string",
        category: BibtexEntryTypeCategory::String,
        documentation: None,
    },
    BibtexEntryType {
        name: "comment",
        category: BibtexEntryTypeCategory::Misc,
        documentation: None,
    },
    BibtexEntryType {
        name: "article",
        category: BibtexEntryTypeCategory::Article,
        documentation: Some("An article in a journal, magazine, newspaper, or other periodical which forms a \n self-contained unit with its own title. The title of the periodical is given in the \n journaltitle field. If the issue has its own title in addition to the main title of \n the periodical, it goes in the issuetitle field. Note that editor and related \n fields refer to the journal while translator and related fields refer to the article.\n\nRequired fields: `author`, `title`, `journaltitle`, `year/date`"),
    },
    BibtexEntryType {
        name: "book",
        category: BibtexEntryTypeCategory::Book,
        documentation: Some("A single-volume book with one or more authors where the authors share credit for\n the work as a whole. This entry type also covers the function of the `@inbook` type\n of traditional BibTeX.\n\nRequired fields: `author`, `title`, `year/date`"),
    },
    BibtexEntryType {
        name: "mvbook",
        category: BibtexEntryTypeCategory::Book,
        documentation: Some("A multi-volume `@book`. For backwards compatibility, multi-volume books are also\n supported by the entry type `@book`. However, it is advisable to make use of the\n dedicated entry type `@mvbook`.\n\nRequired fields: `author`, `title`, `year/date`"),
    },
    BibtexEntryType {
        name: "inbook",
        category: BibtexEntryTypeCategory::Part,
        documentation: Some("A part of a book which forms a self-contained unit with its own title. Note that the\n profile of this entry type is different from standard BibTeX.\n\nRequired fields: `author`, `title`, `booktitle`, `year/date`"),
    },
    BibtexEntryType {
        name: "bookinbook",
        category: BibtexEntryTypeCategory::Part,
        documentation: Some("This type is similar to `@inbook` but intended for works originally published as a\n stand-alone book. A typical example are books reprinted in the collected works of\n an author."),
    },
    BibtexEntryType {
        name: "suppbook",
        category: BibtexEntryTypeCategory::Book,
        documentation: Some("Supplemental material in a `@book`. This type is closely related to the `@inbook`\n entry type. While `@inbook` is primarily intended for a part of a book with its own\n title (e. g., a single essay in a collection of essays by the same author), this type is\n provided for elements such as prefaces, introductions, forewords, afterwords, etc.\n which often have a generic title only. Style guides may require such items to be\n formatted differently from other `@inbook` items. The standard styles will treat this\n entry type as an alias for `@inbook`."),
    },
    BibtexEntryType {
        name: "booklet",
        category: BibtexEntryTypeCategory::Book,
        documentation: Some("A book-like work without a formal publisher or sponsoring institution. Use the field\n howpublished to supply publishing information in free format, if applicable. The\n field type may be useful as well.\n\nRequired fields: `author/editor`, `title`, `year/date`"),
    },
    BibtexEntryType {
        name: "collection",
        category: BibtexEntryTypeCategory::Collection,
        documentation: Some("A single-volume collection with multiple, self-contained contributions by distinct\n authors which have their own title. The work as a whole has no overall author but it\n will usually have an editor.\n\nRequired fields: `editor`, `title`, `year/date`"),
    },
    BibtexEntryType {
        name: "mvcollection",
        category: BibtexEntryTypeCategory::Collection,
        documentation: Some("A multi-volume `@collection`. For backwards compatibility, multi-volume collections\n are also supported by the entry type `@collection`. However, it is advisable\n to make use of the dedicated entry type `@mvcollection`.\n\nRequired fields: `editor`, `title`, `year/date`"),
    },
    BibtexEntryType {
        name: "incollection",
        category: BibtexEntryTypeCategory::Part,
        documentation: Some("A contribution to a collection which forms a self-contained unit with a distinct author\n and title. The `author` refers to the `title`, the `editor` to the `booktitle`, i. e.,\n the title of the collection.\n\nRequired fields: `author`, `title`, `booktitle`, `year/date`"),
    },
    BibtexEntryType {
        name: "suppcollection",
        category: BibtexEntryTypeCategory::Collection,
        documentation: Some("Supplemental material in a `@collection`. This type is similar to `@suppbook` but\n related to the `@collection` entry type. The standard styles will treat this entry\n type as an alias for `@incollection`."),
    },
    BibtexEntryType {
        name: "manual",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Technical or other documentation, not necessarily in printed form. The author or\n editor is omissible.\n\nRequired fields: `author/editor`, `title`, `year/date`"),
    },
    BibtexEntryType {
        name: "misc",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("A fallback type for entries which do not fit into any other category. Use the field\n howpublished to supply publishing information in free format, if applicable. The\n field type may be useful as well. author, editor, and year are omissible.\n\nRequired fields: `author/editor`, `title`, `year/date`"),
    },
    BibtexEntryType {
        name: "online",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("An online resource. `author`, `editor`, and `year` are omissible.\n This entry type is intended for sources such as web sites which are intrinsically\n online resources. Note that all entry types support the url field. For example, when\n adding an article from an online journal, it may be preferable to use the `@article`\n type and its url field.\n\nRequired fields: `author/editor`, `title`, `year/date`, `url`"),
    },
    BibtexEntryType {
        name: "patent",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("A patent or patent request. The number or record token is given in the number\n field. Use the type field to specify the type and the location field to indicate the\n scope of the patent, if different from the scope implied by the type. Note that the\n location field is treated as a key list with this entry type.\n\nRequired fields: `author`, `title`, `number`, `year/date`"),
    },
    BibtexEntryType {
        name: "periodical",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("An complete issue of a periodical, such as a special issue of a journal. The title of\n the periodical is given in the title field. If the issue has its own title in addition to\n the main title of the periodical, it goes in the issuetitle field. The editor is\n omissible.\n\nRequired fields: `editor`, `title`, `year/date`"),
    },
    BibtexEntryType {
        name: "suppperiodical",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Supplemental material in a `@periodical`. This type is similar to `@suppbook`\n but related to the `@periodical` entry type. The role of this entry type may be\n more obvious if you bear in mind that the `@article` type could also be called\n `@inperiodical`. This type may be useful when referring to items such as regular\n columns, obituaries, letters to the editor, etc. which only have a generic title. Style\n guides may require such items to be formatted differently from articles in the strict\n sense of the word. The standard styles will treat this entry type as an alias for\n `@article`."),
    },
    BibtexEntryType {
        name: "proceedings",
        category: BibtexEntryTypeCategory::Book,
        documentation: Some("A single-volume conference proceedings. This type is very similar to `@collection`.\n It supports an optional organization field which holds the sponsoring institution.\n The editor is omissible.\n\nRequired fields: `title`, `year/date`"),
    },
    BibtexEntryType {
        name: "mvproceedings",
        category: BibtexEntryTypeCategory::Book,
        documentation: Some("A multi-volume `@proceedings` entry. For backwards compatibility, multi-volume\n proceedings are also supported by the entry type `@proceedings`. However, it is\n advisable to make use of the dedicated entry type `@mvproceedings`\n\nRequired fields: `title`, `year/date`"),
    },
    BibtexEntryType {
        name: "inproceedings",
        category: BibtexEntryTypeCategory::Part,
        documentation: Some("An article in a conference proceedings. This type is similar to `@incollection`. It\n supports an optional `organization` field.\n\nRequired fields: `author`, `title`, `booktitle`, `year/date`"),
    },
    BibtexEntryType {
        name: "reference",
        category: BibtexEntryTypeCategory::Collection,
        documentation: Some("A single-volume work of reference such as an encyclopedia or a dictionary. This is a\n more specific variant of the generic `@collection` entry type. The standard styles\n will treat this entry type as an alias for `@collection`."),
    },
    BibtexEntryType {
        name: "mvreference",
        category: BibtexEntryTypeCategory::Collection,
        documentation: Some("A multi-volume `@reference` entry. The standard styles will treat this entry type\n as an alias for `@mvcollection`. For backwards compatibility, multi-volume references\n are also supported by the entry type `@reference`. However, it is advisable\n to make use of the dedicated entry type `@mvreference`."),
    },
    BibtexEntryType {
        name: "inreference",
        category: BibtexEntryTypeCategory::Part,
        documentation: Some("An article in a work of reference. This is a more specific variant of the generic\n `@incollection` entry type. The standard styles will treat this entry type as an\n alias for `@incollection`."),
    },
    BibtexEntryType {
        name: "report",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("A technical report, research report, or white paper published by a university or some\n other institution. Use the `type` field to specify the type of report. The sponsoring\n institution goes in the `institution` field.\n\nRequired fields: `author`, `title`, `type`, `institution`, `year/date`"),
    },
    BibtexEntryType {
        name: "set",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("An entry set. This entry type is special."),
    },
    BibtexEntryType {
        name: "thesis",
        category: BibtexEntryTypeCategory::Thesis,
        documentation: Some("A thesis written for an educational institution to satisfy the requirements for a degree.\n Use the `type` field to specify the type of thesis.\n\nRequired fields: `author`, `title`, `type`, `institution`, `year/date`"),
    },
    BibtexEntryType {
        name: "unpublished",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("A work with an author and a title which has not been formally published, such as\n a manuscript or the script of a talk. Use the fields `howpublished` and `note` to\n supply additional information in free format, if applicable.\n\nRequired fields: `author`, `title`, `year/date`"),
    },
    BibtexEntryType {
        name: "xdata",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("This entry type is special. `@xdata` entries hold data which may be inherited by other\n entries using the `xdata` field. Entries of this type only serve as data containers;\n they may not be cited or added to the bibliography."),
    },
    BibtexEntryType {
        name: "conference",
        category: BibtexEntryTypeCategory::Part,
        documentation: Some("A legacy alias for `@inproceedings`."),
    },
    BibtexEntryType {
        name: "electronic",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("An alias for `@online`."),
    },
    BibtexEntryType {
        name: "mastersthesis",
        category: BibtexEntryTypeCategory::Thesis,
        documentation: Some("Similar to `@thesis` except that the `type` field is optional and defaults to the\n localised term ‘Master’s thesis’. You may still use the `type` field to override that."),
    },
    BibtexEntryType {
        name: "phdthesis",
        category: BibtexEntryTypeCategory::Thesis,
        documentation: Some("Similar to `@thesis` except that the `type` field is optional and defaults to the\n localised term ‘PhD thesis’. You may still use the `type` field to override that."),
    },
    BibtexEntryType {
        name: "techreport",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Similar to `@report` except that the `type` field is optional and defaults to the\n localised term ‘technical report’. You may still use the `type` field to override that."),
    },
    BibtexEntryType {
        name: "www",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("An alias for `@online`, provided for `jurabib` compatibility."),
    },
    BibtexEntryType {
        name: "artwork",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Works of the visual arts such as paintings, sculpture, and installations."),
    },
    BibtexEntryType {
        name: "audio",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Audio recordings, typically on audio cd, dvd, audio cassette, or similar media. See\n also `@music`."),
    },
    BibtexEntryType {
        name: "bibnote",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("This special entry type is not meant to be used in the `bib` file like other types. It is\n provided for third-party packages like `notes2bib` which merge notes into the bibliography.\n The notes should go into the `note` field. Be advised that the `@bibnote`\n type is not related to the `defbibnote` command in any way. `defbibnote`\n is for adding comments at the beginning or the end of the bibliography, whereas\n the `@bibnote` type is meant for packages which render endnotes as bibliography\n entries."),
    },
    BibtexEntryType {
        name: "commentary",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Commentaries which have a status different from regular books, such as legal commentaries."),
    },
    BibtexEntryType {
        name: "image",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Images, pictures, photographs, and similar media."),
    },
    BibtexEntryType {
        name: "jurisdiction",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Court decisions, court recordings, and similar things."),
    },
    BibtexEntryType {
        name: "legislation",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Laws, bills, legislative proposals, and similar things."),
    },
    BibtexEntryType {
        name: "legal",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Legal documents such as treaties."),
    },
    BibtexEntryType {
        name: "letter",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Personal correspondence such as letters, emails, memoranda, etc."),
    },
    BibtexEntryType {
        name: "movie",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Motion pictures. See also `@video`."),
    },
    BibtexEntryType {
        name: "music",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Musical recordings. This is a more specific variant of `@audio`."),
    },
    BibtexEntryType {
        name: "performance",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Musical and theatrical performances as well as other works of the performing arts.\n This type refers to the event as opposed to a recording, a score, or a printed play."),
    },
    BibtexEntryType {
        name: "review",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Reviews of some other work. This is a more specific variant of the `@article` type.\n The standard styles will treat this entry type as an alias for `@article`."),
    },
    BibtexEntryType {
        name: "software",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Computer software."),
    },
    BibtexEntryType {
        name: "standard",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("National and international standards issued by a standards body such as the International\n Organization for Standardization."),
    },
    BibtexEntryType {
        name: "video",
        category: BibtexEntryTypeCategory::Misc,
        documentation: Some("Audiovisual recordings, typically on dvd, vhs cassette, or similar media. See also\n `@movie`."),
    }
];

pub static BIBTEX_FIELD_TYPES: &[BibtexFieldType<'static>] = &[
    BibtexFieldType {
        name: "abstract",
        documentation: "This field is intended for recording abstracts in a bib file, to be printed by a special bibliography style. It is not used by all standard bibliography styles.",
    },
    BibtexFieldType {
        name: "addendum",
        documentation: "Miscellaneous bibliographic data to be printed at the end of the entry. This is similar to the `note` field except that it is printed at the end of the bibliography entry.",
    },
    BibtexFieldType {
        name: "afterword",
        documentation: "The author(s) of an afterword to the work. If the author of the afterword is identical to the `editor` and/or `translator`, the standard styles will automatically concatenate these fields in the bibliography. See also `introduction` and `foreword`.",
    },
    BibtexFieldType {
        name: "annotation",
        documentation: "This field may be useful when implementing a style for annotated bibliographies. It is not used by all standard bibliography styles. Note that this field is completely unrelated to `annotator`. The `annotator` is the author of annotations which are part of the work cited.",
    },
    BibtexFieldType {
        name: "annotator",
        documentation: "The author(s) of annotations to the work. If the annotator is identical to the `editor` and/or `translator`, the standard styles will automatically concatenate these fields in the bibliography. See also `commentator`.",
    },
    BibtexFieldType {
        name: "author",
        documentation: "The author(s) of the `title`.",
    },
    BibtexFieldType {
        name: "authortype",
        documentation: "The type of author. This field will affect the string (if any) used to introduce the author. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "bookauthor",
        documentation: "The author(s) of the `booktitle`.",
    },
    BibtexFieldType {
        name: "bookpagination",
        documentation: "If the work is published as part of another one, this is the pagination scheme of the enclosing work, i. e., `bookpagination` relates to `pagination` like `booktitle` to `title`. The value of this field will affect the formatting of the `pages` and `pagetotal` fields. The key should be given in the singular form. Possible keys are `page`, `column`, `line`, `verse`, `section`, and `paragraph`. See also `pagination`.",
    },
    BibtexFieldType {
        name: "booksubtitle",
        documentation: "The subtitle related to the `booktitle`. If the subtitle field refers to a work which is part of a larger publication, a possible subtitle of the main work is given in this field. See also `subtitle`.",
    },
    BibtexFieldType {
        name: "booktitle",
        documentation: "If the `title` field indicates the title of a work which is part of a larger publication, the title of the main work is given in this field. See also `title`.",
    },
    BibtexFieldType {
        name: "booktitleaddon",
        documentation: "An annex to the `booktitle`, to be printed in a different font.",
    },
    BibtexFieldType {
        name: "chapter",
        documentation: "A chapter or section or any other unit of a work.",
    },
    BibtexFieldType {
        name: "commentator",
        documentation: "The author(s) of a commentary to the work. Note that this field is intended for commented editions which have a commentator in addition to the author. If the work is a stand-alone commentary, the commentator should be given in the `author` field. If the commentator is identical to the `editor` and/or `translator`, the standard styles will automatically concatenate these fields in the bibliography. See also `annotator`.",
    },
    BibtexFieldType {
        name: "date",
        documentation: "The publication date. See also `month` and `year`.",
    },
    BibtexFieldType {
        name: "doi",
        documentation: "The Digital Object Identifier of the work.",
    },
    BibtexFieldType {
        name: "edition",
        documentation: "The edition of a printed publication. This must be an integer, not an ordinal. Don’t say `edition={First}` or `edition={1st}` but `edition={1}`. The bibliography style converts this to a language dependent ordinal. It is also possible to give the edition as a literal string, for example \"Third, revised and expanded edition\".",
    },
    BibtexFieldType {
        name: "editor",
        documentation: "The editor(s) of the `title`, `booktitle`, or `maintitle`, depending on the entry type. Use the `editortype` field to specify the role if it is different from `editor`.",
    },
    BibtexFieldType {
        name: "editora",
        documentation: "A secondary editor performing a different editorial role, such as compiling, redacting, etc. Use the `editoratype` field to specify the role.",
    },
    BibtexFieldType {
        name: "editorb",
        documentation: "Another secondary editor performing a different role. Use the `editorbtype` field to specify the role.",
    },
    BibtexFieldType {
        name: "editorc",
        documentation: "Another secondary editor performing a different role. Use the `editorctype` field to specify the role.",
    },
    BibtexFieldType {
        name: "editortype",
        documentation: "The type of editorial role performed by the `editor`. Roles supported by default are `editor`, `compiler`, `founder`, `continuator`, `redactor`, `reviser`, `collaborator`, `organizer`. The role `editor` is the default. In this case, the field is omissible.",
    },
    BibtexFieldType {
        name: "editoratype",
        documentation: "Similar to `editortype` but referring to the `editora` field.",
    },
    BibtexFieldType {
        name: "editorbtype",
        documentation: "Similar to `editortype` but referring to the `editorb` field.",
    },
    BibtexFieldType {
        name: "editorctype",
        documentation: "Similar to `editortype` but referring to the `editorc` field.",
    },
    BibtexFieldType {
        name: "eid",
        documentation: "The electronic identifier of an `@article`.",
    },
    BibtexFieldType {
        name: "entrysubtype",
        documentation: "This field, which is not used by the standard styles, may be used to specify a subtype of an entry type. This may be useful for bibliography styles which support a finergrained set of entry types.",
    },
    BibtexFieldType {
        name: "eprint",
        documentation: "The electronic identifier of an online publication. This is roughly comparable to a doi but specific to a certain archive, repository, service, or system. See also `eprinttype` and `eprintclass`.",
    },
    BibtexFieldType {
        name: "eprintclass",
        documentation: "Additional information related to the resource indicated by the `eprinttype` field. This could be a section of an archive, a path indicating a service, a classification of some sort, etc. See also`eprint` and `eprinttype`.",
    },
    BibtexFieldType {
        name: "eprinttype",
        documentation: "The type of `eprint` identifier, e. g., the name of the archive, repository, service, or system the `eprint` field refers to. See also `eprint` and `eprintclass`.",
    },
    BibtexFieldType {
        name: "eventdate",
        documentation: "The date of a conference, a symposium, or some other event in `@proceedings` and `@inproceedings` entries. See also `eventtitle` and `venue`.",
    },
    BibtexFieldType {
        name: "eventtitle",
        documentation: "The title of a conference, a symposium, or some other event in `@proceedings` and `@inproceedings` entries. Note that this field holds the plain title of the event. Things like \"Proceedings of the Fifth XYZ Conference\" go into the `titleaddon` or `booktitleaddon` field, respectively. See also `eventdate` and `venue`.",
    },
    BibtexFieldType {
        name: "eventtitleaddon",
        documentation: "An annex to the `eventtitle` field. Can be used for known event acronyms, for example.",
    },
    BibtexFieldType {
        name: "file",
        documentation: "A local link to a PDF or other version of the work. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "foreword",
        documentation: "The author(s) of a foreword to the work. If the author of the foreword is identical to the `editor` and/or `translator`, the standard styles will automatically concatenate these fields in the bibliography. See also `introduction` and `afterword`.",
    },
    BibtexFieldType {
        name: "holder",
        documentation: "The holder(s) of a `@patent`, if different from the `author`. Note that corporate holders need to be wrapped in an additional set of braces.",
    },
    BibtexFieldType {
        name: "howpublished",
        documentation: "A publication notice for unusual publications which do not fit into any of the common categories.",
    },
    BibtexFieldType {
        name: "indextitle",
        documentation: "A title to use for indexing instead of the regular `title` field. This field may be useful if you have an entry with a title like \"An Introduction to …\" and want that indexed as \"Introduction to …, An\". Style authors should note that `biblatex` automatically copies the value of the `title` field to `indextitle` if the latter field is undefined.",
    },
    BibtexFieldType {
        name: "institution",
        documentation: "The name of a university or some other institution, depending on the entry type. Traditional BibTeX uses the field name `school` for theses, which is supported as an alias.",
    },
    BibtexFieldType {
        name: "introduction",
        documentation: "The author(s) of an introduction to the work. If the author of the introduction is identical to the `editor` and/or `translator`, the standard styles will automatically concatenate these fields in the bibliography. See also `foreword` and `afterword`.",
    },
    BibtexFieldType {
        name: "isan",
        documentation: "The International Standard Audiovisual Number of an audiovisual work. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "isbn",
        documentation: "The International Standard Book Number of a book.",
    },
    BibtexFieldType {
        name: "ismn",
        documentation: "The International Standard Music Number for printed music such as musical scores. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "isrn",
        documentation: "The International Standard Technical Report Number of a technical report.",
    },
    BibtexFieldType {
        name: "issn",
        documentation: "The International Standard Serial Number of a periodical.",
    },
    BibtexFieldType {
        name: "issue",
        documentation: "The issue of a journal. This field is intended for journals whose individual issues are identified by a designation such as ‘Spring’ or ‘Summer’ rather than the month or a number. The placement of `issue` is similar to `month` and `number`, integer ranges and short designators are better written to the number field. See also `month` and `number`.",
    },
    BibtexFieldType {
        name: "issuesubtitle",
        documentation: "The subtitle of a specific issue of a journal or other periodical.",
    },
    BibtexFieldType {
        name: "issuetitle",
        documentation: "The title of a specific issue of a journal or other periodical.",
    },
    BibtexFieldType {
        name: "iswc",
        documentation: "The International Standard Work Code of a musical work. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "journalsubtitle",
        documentation: "The subtitle of a journal, a newspaper, or some other periodical.",
    },
    BibtexFieldType {
        name: "journaltitle",
        documentation: "The name of a journal, a newspaper, or some other periodical.",
    },
    BibtexFieldType {
        name: "label",
        documentation: "A designation to be used by the citation style as a substitute for the regular label if any data required to generate the regular label is missing. For example, when an author-year citation style is generating a citation for an entry which is missing the author or the year, it may fall back to `label`. Note that, in contrast to `shorthand`, `label` is only used as a fallback. See also `shorthand`.",
    },
    BibtexFieldType {
        name: "language",
        documentation: "The language(s) of the work. Languages may be specified literally or as localisation keys. If localisation keys are used, the prefix lang is omissible. See also `origlanguage`.",
    },
    BibtexFieldType {
        name: "library",
        documentation: "This field may be useful to record information such as a library name and a call number. This may be printed by a special bibliography style if desired. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "location",
        documentation: "The place(s) of publication, i. e., the location of the `publisher` or `institution`, depending on the entry type. Traditional BibTeX uses the field name `address`, which is supported as an alias. With `@patent` entries, this list indicates the scope of a patent.",
    },
    BibtexFieldType {
        name: "mainsubtitle",
        documentation: "The subtitle related to the `maintitle`. See also `subtitle`.",
    },
    BibtexFieldType {
        name: "maintitle",
        documentation: "The main title of a multi-volume book, such as *Collected Works*. If the `title` or `booktitle` field indicates the title of a single volume which is part of multi-volume book, the title of the complete work is given in this field.",
    },
    BibtexFieldType {
        name: "maintitleaddon",
        documentation: "An annex to the `maintitle`, to be printed in a different font.",
    },
    BibtexFieldType {
        name: "month",
        documentation: "The publication month. This must be an integer, not an ordinal or a string. Don’t say `month={January}` but `month={1}`. The bibliography style converts this to a language dependent string or ordinal where required. This field is a literal field only when given explicitly in the data (for plain BibTeX compatibility for example). It is however better to use the `date` field as this supports many more features.",
    },
    BibtexFieldType {
        name: "nameaddon",
        documentation: "An addon to be printed immediately after the author name in the bibliography. Not used by the standard bibliography styles. This field may be useful to add an alias or pen name (or give the real name if the pseudonym is commonly used to refer to that author).",
    },
    BibtexFieldType {
        name: "note",
        documentation: "Miscellaneous bibliographic data which does not fit into any other field. The note field may be used to record bibliographic data in a free format. Publication facts such as \"Reprint of the edition London 1831\" are typical candidates for the note field. See also `addendum`.",
    },
    BibtexFieldType {
        name: "number",
        documentation: "The number of a journal or the volume/number of a book in a `series`. See also `issue`. With `@patent` entries, this is the number or record token of a patent or patent request. Normally this field will be an integer or an integer range, but in certain cases it may also contain \"S1\", \"Suppl. 1\", in these cases the output should be scrutinised carefully.",
    },
    BibtexFieldType {
        name: "organization",
        documentation: "The organization(s) that published a `@manual` or an `@online` resource, or sponsored a conference.",
    },
    BibtexFieldType {
        name: "origdate",
        documentation: "If the work is a translation, a reprint, or something similar, the publication date of the original edition. Not used by the standard bibliography styles. See also `date`.",
    },
    BibtexFieldType {
        name: "origlanguage",
        documentation: "If the work is a translation, the language(s) of the original work. See also `language`.",
    },
    BibtexFieldType {
        name: "origlocation",
        documentation: "If the work is a translation, a reprint, or something similar, the location of the original edition. Not used by the standard bibliography styles. See also `location`.",
    },
    BibtexFieldType {
        name: "origpublisher",
        documentation: "If the work is a translation, a reprint, or something similar, the publisher of the original edition. Not used by the standard bibliography styles. See also `publisher`.",
    },
    BibtexFieldType {
        name: "origtitle",
        documentation: "If the work is a translation, the `title` of the original work. Not used by the standard bibliography styles. See also `title`.",
    },
    BibtexFieldType {
        name: "pages",
        documentation: "One or more page numbers or page ranges. If the work is published as part of another one, such as an article in a journal or a collection, this field holds the relevant page range in that other work. It may also be used to limit the reference to a specific part of a work (a chapter in a book, for example).",
    },
    BibtexFieldType {
        name: "pagetotal",
        documentation: "The total number of pages of the work.",
    },
    BibtexFieldType {
        name: "pagination",
        documentation: "The pagination of the work. The value of this field will affect the formatting the *postnote* argument to a citation command. The key should be given in the singular form. Possible keys are `page`, `column`, `line`, `verse`, `section`, and `paragraph`. See also `bookpagination`.",
    },
    BibtexFieldType {
        name: "part",
        documentation: "The number of a partial volume. This field applies to books only, not to journals. It may be used when a logical volume consists of two or more physical ones. In this case the number of the logical volume goes in the `volume` field and the number of the part of that volume in the `part` field. See also `volume`.",
    },
    BibtexFieldType {
        name: "publisher",
        documentation: "The name(s) of the publisher(s).",
    },
    BibtexFieldType {
        name: "pubstate",
        documentation: "The publication state of the work, e. g., 'in press'.",
    },
    BibtexFieldType {
        name: "reprinttitle",
        documentation: "The title of a reprint of the work. Not used by the standard styles.",
    },
    BibtexFieldType {
        name: "series",
        documentation: "The name of a publication series, such as \"Studies in …\", or the number of a journal series. Books in a publication series are usually numbered. The number or volume of a book in a series is given in the `number` field. Note that the `@article` entry type makes use of the `series` field as well, but handles it in a special way.",
    },
    BibtexFieldType {
        name: "shortauthor",
        documentation: "The author(s) of the work, given in an abbreviated form. This field is mainly intended for abbreviated forms of corporate authors.",
    },
    BibtexFieldType {
        name: "shorteditor",
        documentation: "The editor(s) of the work, given in an abbreviated form. This field is mainly intended for abbreviated forms of corporate editors.",
    },
    BibtexFieldType {
        name: "shorthand",
        documentation: "A special designation to be used by the citation style instead of the usual label. If defined, it overrides the default label. See also `label`.",
    },
    BibtexFieldType {
        name: "shorthandintro",
        documentation: "The verbose citation styles which comes with this package use a phrase like \"henceforth cited as [shorthand]\" to introduce shorthands on the first citation. If the `shorthandintro` field is defined, it overrides the standard phrase. Note that the alternative phrase must include the shorthand.",
    },
    BibtexFieldType {
        name: "shortjournal",
        documentation: "A short version or an acronym of the `journaltitle`. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "shortseries",
        documentation: "A short version or an acronym of the `series` field. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "shorttitle",
        documentation: "The title in an abridged form. This field is usually not included in the bibliography. It is intended for citations in author-title format. If present, the author-title citation styles use this field instead of `title`.",
    },
    BibtexFieldType {
        name: "subtitle",
        documentation: "The subtitle of the work.",
    },
    BibtexFieldType {
        name: "title",
        documentation: "The title of the work.",
    },
    BibtexFieldType {
        name: "titleaddon",
        documentation: "An annex to the `title`, to be printed in a different font.",
    },
    BibtexFieldType {
        name: "translator",
        documentation: "The translator(s) of the `title` or `booktitle`, depending on the entry type. If the translator is identical to the `editor`, the standard styles will automatically concatenate these fields in the bibliography.",
    },
    BibtexFieldType {
        name: "type",
        documentation: "The type of a `manual`, `patent`, `report`, or `thesis`.",
    },
    BibtexFieldType {
        name: "url",
        documentation: "The URL of an online publication. If it is not URL-escaped (no ‘%’ chars) it will be URI-escaped according to RFC 3987, that is, even Unicode chars will be correctly escaped.",
    },
    BibtexFieldType {
        name: "urldate",
        documentation: "The access date of the address specified in the `url` field.",
    },
    BibtexFieldType {
        name: "venue",
        documentation: "The location of a conference, a symposium, or some other event in `@proceedings` and `@inproceedings` entries. Note that the `location` list holds the place of publication. It therefore corresponds to the `publisher` and `institution` lists. The location of the event is given in the `venue` field. See also `eventdate` and `eventtitle`.",
    },
    BibtexFieldType {
        name: "version",
        documentation: "The revision number of a piece of software, a manual, etc.",
    },
    BibtexFieldType {
        name: "volume",
        documentation: "The volume of a multi-volume book or a periodical. It is expected to be an integer, not necessarily in arabic numerals since `biber` will automatically from roman numerals or arabic letter to integers internally for sorting purposes. See also `part`. See the `noroman` option which can be used to suppress roman numeral parsing. This can help in cases where there is an ambiguity between parsing as roman numerals or alphanumeric (e.g. ‘C’).",
    },
    BibtexFieldType {
        name: "volumes",
        documentation: "The total number of volumes of a multi-volume work. Depending on the entry type, this field refers to `title` or `maintitle`. It is expected to be an integer, not necessarily in arabic numerals since `biber` will automatically from roman numerals or arabic letter to integers internally for sorting purposes. See the `noroman` option which can be used to suppress roman numeral parsing. This can help in cases where there is an ambiguity between parsing as roman numerals or alphanumeric (e.g. ‘C’).",
    },
    BibtexFieldType {
        name: "year",
        documentation: "The year of publication. This field is a literal field only when given explicitly in the data (for plain BibTeX compatibility for example). It is however better to use the `date` field as this is compatible with plain years too and supports many more features.",
    },
    BibtexFieldType {
        name: "crossref",
        documentation: "This field holds an entry key for the cross-referencing feature. Child entries with a `crossref` field inherit data from the parent entry specified in the `crossref` field. If the number of child entries referencing a specific parent entry hits a certain threshold, the parent entry is automatically added to the bibliography even if it has not been cited explicitly. The threshold is settable with the `mincrossrefs` package option. Style authors should note that whether or not the `crossref` fields of the child entries are defined on the `biblatex` level depends on the availability of the parent entry. If the parent entry is available, the `crossref` fields of the child entries will be defined. If not, the child entries still inherit the data from the parent entry but their `crossref` fields will be undefined. Whether the parent entry is added to the bibliography implicitly because of the threshold or explicitly because it has been cited does not matter. See also the `xref` field.",
    },
    BibtexFieldType {
        name: "entryset",
        documentation: "This field is specific to entry sets. This field is consumed by the backend processing and does not appear in the `.bbl`.",
    },
    BibtexFieldType {
        name: "execute",
        documentation: "A special field which holds arbitrary TeX code to be executed whenever the data of the respective entry is accessed. This may be useful to handle special cases. Conceptually, this field is comparable to the hooks `AtEveryBibitem`, `AtEveryLositem`, and `AtEveryCitekey`, except that it is definable on a per-entry basis in the `bib` file. Any code in this field is executed automatically immediately after these hooks.",
    },
    BibtexFieldType {
        name: "gender",
        documentation: "The gender of the author or the gender of the editor, if there is no author. The following identifiers are supported: `sf` (feminine singular, a single female name), `sm` (masculine singular, a single male name), `sn` (neuter singular, a single neuter name), `pf` (feminine plural, a list of female names), `pm` (masculine plural, a list of male names), `pn` (neuter plural, a list of neuter names),`pp` (plural, a mixed gender list of names). This information is only required by special bibliography and citation styles and only in certain languages. For example, a citation style may replace recurrent author names with a term such as 'idem'. If the Latin word is used, as is custom in English and French, there is no need to specify the gender. In German publications, however, such key terms are usually given in German and in this case they are gender-sensitive.",
    },
    BibtexFieldType {
        name: "langid",
        documentation: "The language id of the bibliography entry. The alias `hyphenation` is provided for backwards compatibility. The identifier must be a language name known to the `babel/polyglossia` packages. This information may be used to switch hyphenation patterns and localise strings in the bibliography. Note that the language names are case sensitive. The languages currently supported by this package are given in table 2. Note that `babel` treats the identifier `english` as an alias for `british` or `american`, depending on the `babel` version. The `biblatex` package always treats it as an alias for `american`. It is preferable to use the language identifiers `american` and `british` (`babel`) or a language specific option to specify a language variant (`polyglossia`, using the `langidopts` field) to avoid any possible confusion.",
    },
    BibtexFieldType {
        name: "langidopts",
        documentation: "For `polyglossia` users, allows per-entry language specific options. The literal value of this field is passed to `polyglossia`’s language switching facility when using the package option `autolang=langname`.",
    },
    BibtexFieldType {
        name: "ids",
        documentation: "Citation key aliases for the main citation key. An entry may be cited by any of its aliases and `biblatex` will treat the citation as if it had used the primary citation key. This is to aid users who change their citation keys but have legacy documents which use older keys for the same entry. This field is consumed by the backend processing and does not appear in the `.bbl`.",
    },
    BibtexFieldType {
        name: "indexsorttitle",
        documentation: "The title used when sorting the index. In contrast to indextitle, this field is used for sorting only. The printed title in the index is the indextitle or the title field. This field may be useful if the title contains special characters or commands which interfere with the sorting of the index. Style authors should note that biblatex automatically copies the value of either the indextitle or the title field to indexsorttitle if the latter field is undefined.",
    },
    BibtexFieldType {
        name: "keywords",
        documentation: "A separated list of keywords. These keywords are intended for the bibliography filters, they are usually not printed. Note that with the default separator (comma), spaces around the separator are ignored.",
    },
    BibtexFieldType {
        name: "options",
        documentation: "A separated list of entry options in *key*=*value* notation. This field is used to set options on a per-entry basis. Note that citation and bibliography styles may define additional entry options.",
    },
    BibtexFieldType {
        name: "presort",
        documentation: "A special field used to modify the sorting order of the bibliography. This field is the first item the sorting routine considers when sorting the bibliography, hence it may be used to arrange the entries in groups. This may be useful when creating subdivided bibliographies with the bibliography filters. This field is consumed by the backend processing and does not appear in the `.bbl`.",
    },
    BibtexFieldType {
        name: "related",
        documentation: "Citation keys of other entries which have a relationship to this entry. The relationship is specified by the `relatedtype` field.",
    },
    BibtexFieldType {
        name: "relatedoptions",
        documentation: "Per-type options to set for a related entry. Note that this does not set the options on the related entry itself, only the `dataonly` clone which is used as a datasource for the parent entry.",
    },
    BibtexFieldType {
        name: "relatedtype",
        documentation: "An identifier which specified the type of relationship for the keys listed in the `related` field. The identifier is a localised bibliography string printed before the data from the related entry list. It is also used to identify type-specific formatting directives and bibliography macros for the related entries.",
    },
    BibtexFieldType {
        name: "relatedstring",
        documentation: "A field used to override the bibliography string specified by `relatedtype`.",
    },
    BibtexFieldType {
        name: "sortkey",
        documentation: "A field used to modify the sorting order of the bibliography. Think of this field as the master sort key. If present, `biblatex` uses this field during sorting and ignores everything else, except for the presort field. This field is consumed by the backend processing and does not appear in the `.bbl`.",
    },
    BibtexFieldType {
        name: "sortname",
        documentation: "A name or a list of names used to modify the sorting order of the bibliography. If present, this list is used instead of `author` or `editor` when sorting the bibliography. This field is consumed by the backend processing and does not appear in the `.bbl`.",
    },
    BibtexFieldType {
        name: "sortshorthand",
        documentation: "Similar to sortkey but used in the list of shorthands. If present, biblatex uses this field instead of shorthand when sorting the list of shorthands. This is useful if the shorthand field holds shorthands with formatting commands such as `emph` or `\textbf`. This field is consumed by the backend processing and does not appear in the `.bbl`.",
    },
    BibtexFieldType {
        name: "sorttitle",
        documentation: "A field used to modify the sorting order of the bibliography. If present, this field is used instead of the title field when sorting the bibliography. The sorttitle field may come in handy if you have an entry with a title like \"An Introduction to…\" and want that alphabetized under ‘I’ rather than ‘A’. In this case, you could put \"Introduction to…\" in the sorttitle field. This field is consumed by the backend processing and does not appear in the `.bbl`.",
    },
    BibtexFieldType {
        name: "sortyear",
        documentation: "A field used to modify the sorting order of the bibliography. In the default sorting templates, if this field is present, it is used instead of the year field when sorting the bibliography. This field is consumed by the backend processing and does not appear in the `.bbl`.",
    },
    BibtexFieldType {
        name: "xdata",
        documentation: "This field inherits data from one or more `@xdata` entries. Conceptually, the `xdata` field is related to crossref and xref: `crossref` establishes a logical parent/child relation and inherits data; `xref` establishes as logical parent/child relation without inheriting data; `xdata` inherits data without establishing a relation. The value of the `xdata` may be a single entry key or a separated list of keys. This field is consumed by the backend processing and does not appear in the `.bbl`.",
    },
    BibtexFieldType {
        name: "xref",
        documentation: "This field is an alternative cross-referencing mechanism. It differs from `crossref` in that the child entry will not inherit any data from the parent entry specified in the `xref` field. If the number of child entries referencing a specific parent entry hits a certain threshold, the parent entry is automatically added to the bibliography even if it has not been cited explicitly. The threshold is settable with the `minxrefs` package option. Style authors should note that whether or not the `xref` fields of the child entries are defined on the `biblatex` level depends on the availability of the parent entry. If the parent entry is available, the `xref` fields of the child entries will be defined. If not, their `xref` fields will be undefined. Whether the parent entry is added to the bibliography implicitly because of the threshold or explicitly because it has been cited does not matter. See also the `crossref` field.",
    },
    BibtexFieldType {
        name: "namea",
        documentation: "Custom lists for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "nameb",
        documentation: "Custom lists for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "namec",
        documentation: "Custom lists for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "nameatype",
        documentation: "Similar to `authortype` and `editortype` but referring to the fields `name[a--c]`. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "namebtype",
        documentation: "Similar to `authortype` and `editortype` but referring to the fields `name[a--c]`. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "namectype",
        documentation: "Similar to `authortype` and `editortype` but referring to the fields `name[a--c]`. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "lista",
        documentation: "Custom lists for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "listb",
        documentation: "Custom lists for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "listc",
        documentation: "Custom lists for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "listd",
        documentation: "Custom lists for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "liste",
        documentation: "Custom lists for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "listf",
        documentation: "Custom lists for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "usera",
        documentation: "Custom fields for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "userb",
        documentation: "Custom fields for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "userc",
        documentation: "Custom fields for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "userd",
        documentation: "Custom fields for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "usere",
        documentation: "Custom fields for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "userf",
        documentation: "Custom fields for special bibliography styles. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "verba",
        documentation: "Similar to the custom fields except that these are verbatim fields. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "verbb",
        documentation: "Similar to the custom fields except that these are verbatim fields. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "verbc",
        documentation: "Similar to the custom fields except that these are verbatim fields. Not used by the standard bibliography styles.",
    },
    BibtexFieldType {
        name: "address",
        documentation: "An alias for `location`, provided for BibTeX compatibility. Traditional BibTeX uses the slightly misleading field name `address` for the place of publication, i. e., the location of the publisher, while `biblatex` uses the generic field name `location`.",
    },
    BibtexFieldType {
        name: "annote",
        documentation: "An alias for `annotation`, provided for jurabib compatibility.",
    },
    BibtexFieldType {
        name: "archiveprefix",
        documentation: "An alias for `eprinttype`, provided for arXiv compatibility.",
    },
    BibtexFieldType {
        name: "journal",
        documentation: "An alias for `journaltitle`, provided for BibTeX compatibility.",
    },
    BibtexFieldType {
        name: "key",
        documentation: "An alias for `sortkey`, provided for BibTeX compatibility.",
    },
    BibtexFieldType {
        name: "pdf",
        documentation: "An alias for `file`, provided for JabRef compatibility.",
    },
    BibtexFieldType {
        name: "primaryclass",
        documentation: "An alias for `eprintclass`, provided for arXiv compatibility.",
    },
    BibtexFieldType {
        name: "school",
        documentation: "An alias for `institution`, provided for BibTeX compatibility. The `institution` field is used by traditional BibTeX for technical reports whereas the `school` field holds the institution associated with theses. The `biblatex` package employs the generic field name `institution` in both cases.",
    }    
];
