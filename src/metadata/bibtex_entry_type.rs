pub struct BibtexEntryType {
    pub name: &'static str,
    pub documentation: Option<&'static str>,
}

pub fn get_documentation(name: &str) -> Option<&'static str> {
    BIBTEX_ENTRY_TYPES
        .iter()
        .find(|ty| ty.name.to_lowercase() == name.to_lowercase())
        .and_then(|ty| ty.documentation)
}

pub static BIBTEX_ENTRY_TYPES: &'static [BibtexEntryType] = &[
    BibtexEntryType {
        name: "preamble",
        documentation: None
    },
    BibtexEntryType {
        name: "string",
        documentation: None
    },
    BibtexEntryType {
        name: "article",
        documentation: Some("An article in a journal, magazine, newspaper, or other periodical which forms a \n self-contained unit with its own title. The title of the periodical is given in the \n journaltitle field. If the issue has its own title in addition to the main title of \n the periodical, it goes in the issuetitle field. Note that editor and related \n fields refer to the journal while translator and related fields refer to the article.\n\nRequired fields: `author`, `title`, `journaltitle`, `year/date`")
    },
    BibtexEntryType {
        name: "book",
        documentation: Some("A single-volume book with one or more authors where the authors share credit for\n the work as a whole. This entry type also covers the function of the `@inbook` type\n of traditional BibTeX.\n\nRequired fields: `author`, `title`, `year/date`")
    },
    BibtexEntryType {
        name: "mvbook",
        documentation: Some("A multi-volume `@book`. For backwards compatibility, multi-volume books are also\n supported by the entry type `@book`. However, it is advisable to make use of the\n dedicated entry type `@mvbook`.\n\nRequired fields: `author`, `title`, `year/date`")
    },
    BibtexEntryType {
        name: "inbook",
        documentation: Some("A part of a book which forms a self-contained unit with its own title. Note that the\n profile of this entry type is different from standard BibTeX.\n\nRequired fields: `author`, `title`, `booktitle`, `year/date`")
    },
    BibtexEntryType {
        name: "bookinbook",
        documentation: Some("This type is similar to `@inbook` but intended for works originally published as a\n stand-alone book. A typical example are books reprinted in the collected works of\n an author.")
    },
    BibtexEntryType {
        name: "suppbook",
        documentation: Some("Supplemental material in a `@book`. This type is closely related to the `@inbook`\n entry type. While `@inbook` is primarily intended for a part of a book with its own\n title (e. g., a single essay in a collection of essays by the same author), this type is\n provided for elements such as prefaces, introductions, forewords, afterwords, etc.\n which often have a generic title only. Style guides may require such items to be\n formatted differently from other `@inbook` items. The standard styles will treat this\n entry type as an alias for `@inbook`.")
    },
    BibtexEntryType {
        name: "booklet",
        documentation: Some("A book-like work without a formal publisher or sponsoring institution. Use the field\n howpublished to supply publishing information in free format, if applicable. The\n field type may be useful as well.\n\nRequired fields: `author/editor`, `title`, `year/date`")
    },
    BibtexEntryType {
        name: "collection",
        documentation: Some("A single-volume collection with multiple, self-contained contributions by distinct\n authors which have their own title. The work as a whole has no overall author but it\n will usually have an editor.\n\nRequired fields: `editor`, `title`, `year/date`")
    },
    BibtexEntryType {
        name: "mvcollection",
        documentation: Some("A multi-volume `@collection`. For backwards compatibility, multi-volume collections\n are also supported by the entry type `@collection`. However, it is advisable\n to make use of the dedicated entry type `@mvcollection`.\n\nRequired fields: `editor`, `title`, `year/date`")
    },
    BibtexEntryType {
        name: "incollection",
        documentation: Some("A contribution to a collection which forms a self-contained unit with a distinct author\n and title. The `author` refers to the `title`, the `editor` to the `booktitle`, i. e.,\n the title of the collection.\n\nRequired fields: `author`, `title`, `booktitle`, `year/date`")
    },
    BibtexEntryType {
        name: "suppcollection",
        documentation: Some("Supplemental material in a `@collection`. This type is similar to `@suppbook` but\n related to the `@collection` entry type. The standard styles will treat this entry\n type as an alias for `@incollection`.")
    },
    BibtexEntryType {
        name: "manual",
        documentation: Some("Technical or other documentation, not necessarily in printed form. The author or\n editor is omissible.\n\nRequired fields: `author/editor`, `title`, `year/date`")
    },
    BibtexEntryType {
        name: "misc",
        documentation: Some("A fallback type for entries which do not fit into any other category. Use the field\n howpublished to supply publishing information in free format, if applicable. The\n field type may be useful as well. author, editor, and year are omissible.\n\nRequired fields: `author/editor`, `title`, `year/date`")
    },
    BibtexEntryType {
        name: "online",
        documentation: Some("An online resource. `author`, `editor`, and `year` are omissible.\n This entry type is intended for sources such as web sites which are intrinsically\n online resources. Note that all entry types support the url field. For example, when\n adding an article from an online journal, it may be preferable to use the `@article`\n type and its url field.\n\nRequired fields: `author/editor`, `title`, `year/date`, `url`")
    },
    BibtexEntryType {
        name: "patent",
        documentation: Some("A patent or patent request. The number or record token is given in the number\n field. Use the type field to specify the type and the location field to indicate the\n scope of the patent, if different from the scope implied by the type. Note that the\n location field is treated as a key list with this entry type.\n\nRequired fields: `author`, `title`, `number`, `year/date`")
    },
    BibtexEntryType {
        name: "periodical",
        documentation: Some("An complete issue of a periodical, such as a special issue of a journal. The title of\n the periodical is given in the title field. If the issue has its own title in addition to\n the main title of the periodical, it goes in the issuetitle field. The editor is\n omissible.\n\nRequired fields: `editor`, `title`, `year/date`")
    },
    BibtexEntryType {
        name: "suppperiodical",
        documentation: Some("Supplemental material in a `@periodical`. This type is similar to `@suppbook`\n but related to the `@periodical` entry type. The role of this entry type may be\n more obvious if you bear in mind that the `@article` type could also be called\n `@inperiodical`. This type may be useful when referring to items such as regular\n columns, obituaries, letters to the editor, etc. which only have a generic title. Style\n guides may require such items to be formatted differently from articles in the strict\n sense of the word. The standard styles will treat this entry type as an alias for\n `@article`.")
    },
    BibtexEntryType {
        name: "proceedings",
        documentation: Some("A single-volume conference proceedings. This type is very similar to `@collection`.\n It supports an optional organization field which holds the sponsoring institution.\n The editor is omissible.\n\nRequired fields: `title`, `year/date`")
    },
    BibtexEntryType {
        name: "mvproceedings",
        documentation: Some("A multi-volume `@proceedings` entry. For backwards compatibility, multi-volume\n proceedings are also supported by the entry type `@proceedings`. However, it is\n advisable to make use of the dedicated entry type `@mvproceedings`\n\nRequired fields: `title`, `year/date`")
    },
    BibtexEntryType {
        name: "inproceedings",
        documentation: Some("An article in a conference proceedings. This type is similar to `@incollection`. It\n supports an optional `organization` field.\n\nRequired fields: `author`, `title`, `booktitle`, `year/date`")
    },
    BibtexEntryType {
        name: "reference",
        documentation: Some("A single-volume work of reference such as an encyclopedia or a dictionary. This is a\n more specific variant of the generic `@collection` entry type. The standard styles\n will treat this entry type as an alias for `@collection`.")
    },
    BibtexEntryType {
        name: "mvreference",
        documentation: Some("A multi-volume `@reference` entry. The standard styles will treat this entry type\n as an alias for `@mvcollection`. For backwards compatibility, multi-volume references\n are also supported by the entry type `@reference`. However, it is advisable\n to make use of the dedicated entry type `@mvreference`.")
    },
    BibtexEntryType {
        name: "inreference",
        documentation: Some("An article in a work of reference. This is a more specific variant of the generic\n `@incollection` entry type. The standard styles will treat this entry type as an\n alias for `@incollection`.")
    },
    BibtexEntryType {
        name: "report",
        documentation: Some("A technical report, research report, or white paper published by a university or some\n other institution. Use the `type` field to specify the type of report. The sponsoring\n institution goes in the `institution` field.\n\nRequired fields: `author`, `title`, `type`, `institution`, `year/date`")
    },
    BibtexEntryType {
        name: "set",
        documentation: Some("An entry set. This entry type is special.")
    },
    BibtexEntryType {
        name: "thesis",
        documentation: Some("A thesis written for an educational institution to satisfy the requirements for a degree.\n Use the `type` field to specify the type of thesis.\n\nRequired fields: `author`, `title`, `type`, `institution`, `year/date`")
    },
    BibtexEntryType {
        name: "unpublished",
        documentation: Some("A work with an author and a title which has not been formally published, such as\n a manuscript or the script of a talk. Use the fields `howpublished` and `note` to\n supply additional information in free format, if applicable.\n\nRequired fields: `author`, `title`, `year/date`")
    },
    BibtexEntryType {
        name: "xdata",
        documentation: Some("This entry type is special. `@xdata` entries hold data which may be inherited by other\n entries using the `xdata` field. Entries of this type only serve as data containers;\n they may not be cited or added to the bibliography.")
    },
    BibtexEntryType {
        name: "conference",
        documentation: Some("A legacy alias for `@inproceedings`.")
    },
    BibtexEntryType {
        name: "electronic", 
        documentation: Some("An alias for `@online`.")
    },
    BibtexEntryType {
        name: "mastersthesis",
        documentation: Some("Similar to `@thesis` except that the `type` field is optional and defaults to the\n localised term ‘Master’s thesis’. You may still use the `type` field to override that.")
    },
    BibtexEntryType {
        name: "phdthesis",
        documentation: Some("Similar to `@thesis` except that the `type` field is optional and defaults to the\n localised term ‘PhD thesis’. You may still use the `type` field to override that.")
    },
    BibtexEntryType {
        name: "techreport",
        documentation: Some("Similar to `@report` except that the `type` field is optional and defaults to the\n localised term ‘technical report’. You may still use the `type` field to override that.")
    },
    BibtexEntryType {
        name: "www",
        documentation: Some("An alias for `@online`, provided for `jurabib` compatibility.")
    },
    BibtexEntryType {
        name: "artwork",
        documentation: Some("Works of the visual arts such as paintings, sculpture, and installations.")
    },
    BibtexEntryType {
        name: "audio",
        documentation: Some("Audio recordings, typically on audio cd, dvd, audio cassette, or similar media. See\n also `@music`.")
    },
    BibtexEntryType {
        name: "bibnote",
        documentation: Some("This special entry type is not meant to be used in the `bib` file like other types. It is\n provided for third-party packages like `notes2bib` which merge notes into the bibliography.\n The notes should go into the `note` field. Be advised that the `@bibnote`\n type is not related to the `defbibnote` command in any way. `defbibnote`\n is for adding comments at the beginning or the end of the bibliography, whereas\n the `@bibnote` type is meant for packages which render endnotes as bibliography\n entries.")
    },
    BibtexEntryType {
        name: "commentary",
        documentation: Some("Commentaries which have a status different from regular books, such as legal commentaries.")
    },
    BibtexEntryType {
        name: "image",
        documentation: Some("Images, pictures, photographs, and similar media.")
    },
    BibtexEntryType {
        name: "jurisdiction",
        documentation: Some("Court decisions, court recordings, and similar things.")
    },
    BibtexEntryType {
        name: "legislation",
        documentation: Some("Laws, bills, legislative proposals, and similar things.")
    },
    BibtexEntryType {
        name: "legal", 
        documentation: Some("Legal documents such as treaties.")
    },
    BibtexEntryType {
        name: "letter",
        documentation: Some("Personal correspondence such as letters, emails, memoranda, etc.")
    },
    BibtexEntryType {
        name: "movie", 
        documentation: Some("Motion pictures. See also `@video`.")
    },
    BibtexEntryType {
        name: "music",
        documentation: Some("Musical recordings. This is a more specific variant of `@audio`.")
    },
    BibtexEntryType {
        name: "performance",
        documentation: Some("Musical and theatrical performances as well as other works of the performing arts.\n This type refers to the event as opposed to a recording, a score, or a printed play.")
    },
    BibtexEntryType {
        name: "review",
        documentation: Some("Reviews of some other work. This is a more specific variant of the `@article` type.\n The standard styles will treat this entry type as an alias for `@article`.")
    },
    BibtexEntryType {
        name: "software", 
        documentation: Some("Computer software.")
    },
    BibtexEntryType {
        name: "standard",
        documentation: Some("National and international standards issued by a standards body such as the International\n Organization for Standardization.")
    },
    BibtexEntryType {
        name: "video",
        documentation: Some("Audiovisual recordings, typically on dvd, vhs cassette, or similar media. See also\n `@movie`.")
    }
];
