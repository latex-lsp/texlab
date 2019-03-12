import { EOL } from 'os';

export const BIBTEX_TYPES = [
  'preamble',
  'string',
  'article',
  'book',
  'mvbook',
  'inbook',
  'bookinbook',
  'suppbook',
  'booklet',
  'collection',
  'mvcollection',
  'incollection',
  'suppcollection',
  'manual',
  'misc',
  'online',
  'patent',
  'periodical',
  'suppperiodical',
  'proceedings',
  'mvproceedings',
  'inproceedings',
  'reference',
  'mvreference',
  'inreference',
  'report',
  'set',
  'thesis',
  'unpublished',
  'xdata',
  'conference',
  'electronic',
  'mastersthesis',
  'phdthesis',
  'techreport',
  'www',
  'artwork',
  'audio',
  'bibnote',
  'commentary',
  'image',
  'jurisdiction',
  'legislation',
  'legal',
  'letter',
  'movie',
  'music',
  'performance',
  'review',
  'software',
  'standard',
  'video',
];

export function getTypeDocumentation(type: string): string | undefined {
  switch (type) {
    case 'article':
      return (
        `An article in a journal, magazine, newspaper, or other periodical which forms a 
         self-contained unit with its own title. The title of the periodical is given in the 
         journaltitle field. If the issue has its own title in addition to the main title of 
         the periodical, it goes in the issuetitle field. Note that editor and related 
         fields refer to the journal while translator and related fields refer to the article.` +
        EOL +
        EOL +
        'Required fields: `author`, `title`, `journaltitle`, `year/date`'
      );

    case 'book':
      return (
        `A single-volume book with one or more authors where the authors share credit for
         the work as a whole. This entry type also covers the function of the \`@inbook\` type
         of traditional BibTeX.` +
        EOL +
        EOL +
        'Required fields: `author`, `title`, `year/date`'
      );

    case 'mvbook':
      return (
        `A multi-volume \`@book\`. For backwards compatibility, multi-volume books are also
         supported by the entry type \`@book\`. However, it is advisable to make use of the
         dedicated entry type \`@mvbook\`.` +
        EOL +
        EOL +
        'Required fields: `author`, `title`, `year/date`'
      );

    case 'inbook':
      return (
        `A part of a book which forms a self-contained unit with its own title. Note that the
         profile of this entry type is different from standard BibTeX.` +
        EOL +
        EOL +
        'Required fields: `author`, `title`, `booktitle`, `year/date`'
      );

    case 'bookinbook':
      return `This type is similar to \`@inbook\` but intended for works originally published as a
              stand-alone book. A typical example are books reprinted in the collected works of
              an author.`;

    case 'suppbook':
      return `Supplemental material in a \`@book\`. This type is closely related to the \`@inbook\`
              entry type. While \`@inbook\` is primarily intended for a part of a book with its own
              title (e. g., a single essay in a collection of essays by the same author), this type is
              provided for elements such as prefaces, introductions, forewords, afterwords, etc.
              which often have a generic title only. Style guides may require such items to be
              formatted differently from other \`@inbook\` items. The standard styles will treat this
              entry type as an alias for \`@inbook\`.`;

    case 'booklet':
      return (
        `A book-like work without a formal publisher or sponsoring institution. Use the field
         howpublished to supply publishing information in free format, if applicable. The
         field type may be useful as well.` +
        EOL +
        EOL +
        'Required fields: `author/editor`, `title`, `year/date`'
      );

    case 'collection':
      return (
        `A single-volume collection with multiple, self-contained contributions by distinct
         authors which have their own title. The work as a whole has no overall author but it
         will usually have an editor.` +
        EOL +
        EOL +
        'Required fields: `editor`, `title`, `year/date`'
      );

    case 'mvcollection':
      return (
        `A multi-volume \`@collection\`. For backwards compatibility, multi-volume collections
              are also supported by the entry type \`@collection\`. However, it is advisable
              to make use of the dedicated entry type \`@mvcollection\`.` +
        EOL +
        EOL +
        'Required fields: `editor`, `title`, `year/date`'
      );

    case 'incollection':
      return (
        `A contribution to a collection which forms a self-contained unit with a distinct author
              and title. The \`author\` refers to the \`title\`, the \`editor\` to the \`booktitle\`, i. e.,
              the title of the collection.` +
        EOL +
        EOL +
        'Required fields: `author`, `title`, `booktitle`, `year/date`'
      );

    case 'suppcollection':
      return `Supplemental material in a \`@collection\`. This type is similar to \`@suppbook\` but
              related to the \`@collection\` entry type. The standard styles will treat this entry
              type as an alias for \`@incollection\`.`;

    case 'manual':
      return (
        `Technical or other documentation, not necessarily in printed form. The author or
         editor is omissible.` +
        EOL +
        EOL +
        'Required fields: `author/editor`, `title`, `year/date`'
      );

    case 'misc':
      return (
        `A fallback type for entries which do not fit into any other category. Use the field
         howpublished to supply publishing information in free format, if applicable. The
         field type may be useful as well. author, editor, and year are omissible.` +
        EOL +
        EOL +
        'Required fields: `author/editor`, `title`, `year/date`'
      );

    case 'online':
      return (
        `An online resource. \`author\`, \`editor\`, and \`year\` are omissible.
         This entry type is intended for sources such as web sites which are intrinsically
         online resources. Note that all entry types support the url field. For example, when
         adding an article from an online journal, it may be preferable to use the \`@article\`
         type and its url field.` +
        EOL +
        EOL +
        'Required fields: `author/editor`, `title`, `year/date`, `url`'
      );

    case 'patent':
      return (
        `A patent or patent request. The number or record token is given in the number
         field. Use the type field to specify the type and the location field to indicate the
         scope of the patent, if different from the scope implied by the type. Note that the
         location field is treated as a key list with this entry type.` +
        EOL +
        EOL +
        'Required fields: `author`, `title`, `number`, `year/date`'
      );

    case 'periodical':
      return (
        `An complete issue of a periodical, such as a special issue of a journal. The title of
         the periodical is given in the title field. If the issue has its own title in addition to
         the main title of the periodical, it goes in the issuetitle field. The editor is
         omissible.` +
        EOL +
        EOL +
        'Required fields: `editor`, `title`, `year/date`'
      );

    case 'suppperiodical':
      return `Supplemental material in a \`@periodical\`. This type is similar to \`@suppbook\`
              but related to the \`@periodical\` entry type. The role of this entry type may be
              more obvious if you bear in mind that the \`@article\` type could also be called
              \`@inperiodical\`. This type may be useful when referring to items such as regular
              columns, obituaries, letters to the editor, etc. which only have a generic title. Style
              guides may require such items to be formatted differently from articles in the strict
              sense of the word. The standard styles will treat this entry type as an alias for
              \`@article\`.`;

    case 'proceedings':
      return (
        `A single-volume conference proceedings. This type is very similar to \`@collection\`.
              It supports an optional organization field which holds the sponsoring institution.
              The editor is omissible.` +
        EOL +
        EOL +
        'Required fields: `title`, `year/date`'
      );

    case 'mvproceedings':
      return (
        `A multi-volume \`@proceedings\` entry. For backwards compatibility, multi-volume
              proceedings are also supported by the entry type \`@proceedings\`. However, it is
              advisable to make use of the dedicated entry type \`@mvproceedings\`` +
        EOL +
        EOL +
        'Required fields: `title`, `year/date`'
      );

    case 'inproceedings':
      return (
        `An article in a conference proceedings. This type is similar to \`@incollection\`. It
              supports an optional \`organization\` field.` +
        EOL +
        EOL +
        'Required fields: `author`, `title`, `booktitle`, `year/date`'
      );

    case 'reference':
      return `A single-volume work of reference such as an encyclopedia or a dictionary. This is a
              more specific variant of the generic \`@collection\` entry type. The standard styles
              will treat this entry type as an alias for \`@collection\`.`;

    case 'mvreference':
      return `A multi-volume \`@reference\` entry. The standard styles will treat this entry type
              as an alias for \`@mvcollection\`. For backwards compatibility, multi-volume references
              are also supported by the entry type \`@reference\`. However, it is advisable
              to make use of the dedicated entry type \`@mvreference\`.`;

    case 'inreference':
      return `An article in a work of reference. This is a more specific variant of the generic
              \`@incollection\` entry type. The standard styles will treat this entry type as an
              alias for \`@incollection\`.`;

    case 'report':
      return (
        `A technical report, research report, or white paper published by a university or some
              other institution. Use the \`type\` field to specify the type of report. The sponsoring
              institution goes in the \`institution\` field.` +
        EOL +
        EOL +
        'Required fields: `author`, `title`, `type`, `institution`, `year/date`'
      );

    case 'set':
      return `An entry set. This entry type is special.`;

    case 'thesis':
      return (
        `A thesis written for an educational institution to satisfy the requirements for a degree.
         Use the \`type\` field to specify the type of thesis.` +
        EOL +
        EOL +
        'Required fields: `author`, `title`, `type`, `institution`, `year/date`'
      );

    case 'unpublished':
      return (
        `A work with an author and a title which has not been formally published, such as
              a manuscript or the script of a talk. Use the fields \`howpublished\` and \`note\` to
              supply additional information in free format, if applicable.` +
        EOL +
        EOL +
        'Required fields: `author`, `title`, `year/date`'
      );

    case 'xdata':
      return `This entry type is special. \`@xdata\` entries hold data which may be inherited by other
              entries using the \`xdata\` field. Entries of this type only serve as data containers;
              they may not be cited or added to the bibliography.`;

    case 'conference':
      return `A legacy alias for \`@inproceedings\`.`;

    case 'electronic':
      return `An alias for \`@online\`.`;

    case 'mastersthesis':
      return `Similar to \`@thesis\` except that the \`type\` field is optional and defaults to the
              localised term ‘Master’s thesis’. You may still use the \`type\` field to override that.`;

    case 'phdthesis':
      return `Similar to \`@thesis\` except that the \`type\` field is optional and defaults to the
              localised term ‘PhD thesis’. You may still use the \`type\` field to override that.`;

    case 'techreport':
      return `Similar to \`@report\` except that the \`type\` field is optional and defaults to the
              localised term ‘technical report’. You may still use the \`type\` field to override that.`;

    case 'www':
      return `An alias for \`@online\`, provided for \`jurabib\` compatibility.`;

    case 'artwork':
      return `Works of the visual arts such as paintings, sculpture, and installations.`;

    case 'audio':
      return `Audio recordings, typically on audio cd, dvd, audio cassette, or similar media. See
              also \`@music\`.`;

    case 'bibnote':
      return `This special entry type is not meant to be used in the \`bib\` file like other types. It is
              provided for third-party packages like \`notes2bib\` which merge notes into the bibliography.
              The notes should go into the \`note\` field. Be advised that the \`@bibnote\`
              type is not related to the \`\defbibnote\` command in any way. \`\defbibnote\`
              is for adding comments at the beginning or the end of the bibliography, whereas
              the \`@bibnote\` type is meant for packages which render endnotes as bibliography
              entries.`;

    case 'commentary':
      return `Commentaries which have a status different from regular books, such as legal commentaries.`;

    case 'image':
      return `Images, pictures, photographs, and similar media.`;

    case 'jurisdiction':
      return `Court decisions, court recordings, and similar things.`;

    case 'legislation':
      return `Laws, bills, legislative proposals, and similar things.`;

    case 'legal':
      return `Legal documents such as treaties.`;

    case 'letter':
      return `Personal correspondence such as letters, emails, memoranda, etc.`;

    case 'movie':
      return `Motion pictures. See also \`@video\`.`;

    case 'music':
      return `Musical recordings. This is a more specific variant of \`@audio\`.`;

    case 'performance':
      return `Musical and theatrical performances as well as other works of the performing arts.
              This type refers to the event as opposed to a recording, a score, or a printed play.`;

    case 'review':
      return `Reviews of some other work. This is a more specific variant of the \`@article\` type.
              The standard styles will treat this entry type as an alias for \`@article\`.`;

    case 'software':
      return `Computer software.`;

    case 'standard':
      return `National and international standards issued by a standards body such as the International
              Organization for Standardization.`;

    case 'video':
      return `Audiovisual recordings, typically on dvd, vhs cassette, or similar media. See also
              \`@movie\`.`;
    default:
      return undefined;
  }
}
