// Ported from: https://github.com/michel-kraemer/citeproc-java/tree/master/citeproc-java/templates
// Michel Kraemer
// Apache License 2.0
use citeproc_io::{unicode::is_latin_cyrillic, Date, DateOrRange, Name, NumberLike, Reference};
use csl::*;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RisType {
    Abst,
    Advs,
    Aggr,
    Ancient,
    Art,
    Bill,
    Blog,
    Book,
    Case,
    Chap,
    Chart,
    Clswk,
    Comp,
    Conf,
    Cpaper,
    Ctlg,
    Data,
    Dbase,
    Dict,
    Ebook,
    Echap,
    Edbook,
    Ejour,
    Elec,
    Encyc,
    Equa,
    Figure,
    Gen,
    Govdoc,
    Grant,
    Hear,
    Icomm,
    Inpr,
    Jfull,
    Jour,
    Legal,
    Manscpt,
    Map,
    Mgzn,
    Mpct,
    Multi,
    Music,
    News,
    Pamp,
    Pat,
    Pcomm,
    Rprt,
    Ser,
    Slide,
    Sound,
    Stand,
    Stat,
    Std,
    Thes,
    Unpb,
    Video,
}

impl RisType {
    pub fn parse(ty: &str) -> Option<Self> {
        let ty = format!("\"{}\"", ty);
        serde_json::from_str(&ty).ok()
    }

    pub fn csl(self) -> CslType {
        match self {
            Self::Abst => CslType::Article,
            Self::Advs => CslType::Article,
            Self::Aggr => CslType::Dataset,
            Self::Ancient => CslType::Article,
            Self::Art => CslType::Article,
            Self::Bill => CslType::Bill,
            Self::Blog => CslType::Webpage,
            Self::Book => CslType::Book,
            Self::Case => CslType::LegalCase,
            Self::Chap => CslType::Chapter,
            Self::Chart => CslType::Article,
            Self::Clswk => CslType::Article,
            Self::Comp => CslType::Article,
            Self::Conf => CslType::PaperConference,
            Self::Cpaper => CslType::PaperConference,
            Self::Ctlg => CslType::Book,
            Self::Data => CslType::Dataset,
            Self::Dbase => CslType::Dataset,
            Self::Dict => CslType::EntryDictionary,
            Self::Ebook => CslType::Book,
            Self::Echap => CslType::Chapter,
            Self::Edbook => CslType::Book,
            Self::Ejour => CslType::ArticleJournal,
            Self::Elec => CslType::Article,
            Self::Encyc => CslType::EntryEncyclopedia,
            Self::Equa => CslType::Article,
            Self::Figure => CslType::Figure,
            Self::Gen => CslType::Article,
            Self::Govdoc => CslType::Legislation,
            Self::Grant => CslType::Legislation,
            Self::Hear => CslType::Article,
            Self::Icomm => CslType::PersonalCommunication,
            Self::Inpr => CslType::PaperConference,
            Self::Jfull => CslType::ArticleJournal,
            Self::Jour => CslType::ArticleJournal,
            Self::Legal => CslType::Legislation,
            Self::Manscpt => CslType::Manuscript,
            Self::Map => CslType::Map,
            Self::Mgzn => CslType::ArticleMagazine,
            Self::Mpct => CslType::MotionPicture,
            Self::Multi => CslType::Webpage,
            Self::Music => CslType::Song,
            Self::News => CslType::ArticleNewspaper,
            Self::Pamp => CslType::Pamphlet,
            Self::Pat => CslType::Patent,
            Self::Pcomm => CslType::PersonalCommunication,
            Self::Rprt => CslType::Report,
            Self::Ser => CslType::Article,
            Self::Slide => CslType::Article,
            Self::Sound => CslType::Song,
            Self::Stand => CslType::Article,
            Self::Stat => CslType::Legislation,
            Self::Std => CslType::Article,
            Self::Thes => CslType::Thesis,
            Self::Unpb => CslType::Article,
            Self::Video => CslType::MotionPicture,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct RisReference {
    pub id: Option<String>,
    pub ty: Option<RisType>,
    pub editors: Vec<String>,
    pub tertiary_authors: Vec<String>,
    pub subsidiary_authors: Vec<String>,
    pub abstrct: Option<String>,
    pub author_address: Option<String>,
    pub accession_number: Option<String>,
    pub authors: Vec<String>,
    pub book_or_conference: Option<String>,
    pub custom1: Option<String>,
    pub custom2: Option<String>,
    pub custom3: Option<String>,
    pub custom4: Option<String>,
    pub custom5: Option<String>,
    pub custom6: Option<String>,
    pub custom7: Option<String>,
    pub custom8: Option<String>,
    pub caption: Option<String>,
    pub call_number: Option<String>,
    pub place: Option<String>,
    pub date: Option<String>,
    pub name_of_database: Option<String>,
    pub doi: Option<String>,
    pub database_provider: Option<String>,
    pub end_page: Option<String>,
    pub edition: Option<String>,
    pub issue: Option<String>,
    pub journal: Option<String>,
    pub keywords: Vec<String>,
    pub file_attachments: Vec<String>,
    pub figure: Option<String>,
    pub language: Option<String>,
    pub label: Option<String>,
    pub number: Option<String>,
    pub type_of_work: Option<String>,
    pub notes: Vec<String>,
    pub number_of_volumes: Option<String>,
    pub original_publication: Option<String>,
    pub publisher: Option<String>,
    pub year: Option<String>,
    pub reviewed_item: Option<String>,
    pub research_notes: Option<String>,
    pub reprint_edition: Option<String>,
    pub section: Option<String>,
    pub isbn_or_issn: Option<String>,
    pub start_page: Option<String>,
    pub short_title: Option<String>,
    pub primary_title: Option<String>,
    pub secondardy_title: Option<String>,
    pub tertiary_title: Option<String>,
    pub translated_authors: Vec<String>,
    pub title: Option<String>,
    pub translated_title: Option<String>,
    pub url: Option<String>,
    pub volume: Option<String>,
    pub access_date: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RisLibrary {
    pub references: Vec<RisReference>,
}

impl RisLibrary {
    pub fn parse<'a, I: Iterator<Item = &'a str>>(lines: I) -> Self {
        let mut library = Self {
            references: Vec::new(),
        };

        let mut reference: RisReference = RisReference::default();
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line == "ER  -" {
                library.references.push(reference);
                reference = RisReference::default();
                continue;
            }

            let chars: Vec<_> = line.chars().collect();
            if chars.len() < 7 || chars[4] != '-' {
                continue;
            }

            let key: String = (&chars[..2]).iter().collect();
            let value: String = (&chars[6..]).iter().collect();
            match key.to_uppercase().as_str() {
                "TY" => reference.ty = RisType::parse(&value),
                "A2" => reference.editors.push(value),
                "A3" => reference.tertiary_authors.push(value),
                "A4" => reference.subsidiary_authors.push(value),
                "AB" => reference.abstrct = Some(value),
                "AD" => reference.author_address = Some(value),
                "AN" => reference.accession_number = Some(value),
                "AU" => reference.authors.push(value),
                "BT" => reference.book_or_conference = Some(value),
                "C1" => reference.custom1 = Some(value),
                "C2" => reference.custom2 = Some(value),
                "C3" => reference.custom3 = Some(value),
                "C4" => reference.custom4 = Some(value),
                "C5" => reference.custom5 = Some(value),
                "C6" => reference.custom6 = Some(value),
                "C7" => reference.custom7 = Some(value),
                "C8" => reference.custom8 = Some(value),
                "CA" => reference.caption = Some(value),
                "CN" => reference.call_number = Some(value),
                "CY" => reference.place = Some(value),
                "DA" => reference.date = Some(value),
                "DB" => reference.name_of_database = Some(value),
                "DO" => reference.doi = Some(value),
                "DP" => reference.database_provider = Some(value),
                "ED" => reference.editors.push(value),
                "EP" => reference.end_page = Some(value),
                "ET" => reference.edition = Some(value),
                "ID" => reference.id = Some(value),
                "IS" => reference.issue = Some(value),
                "JO" => reference.journal = Some(value),
                "J2" => reference.journal = Some(value),
                "KW" => reference.keywords.push(value),
                "L1" => reference.file_attachments.push(value),
                "L4" => reference.figure = Some(value),
                "LA" => reference.language = Some(value),
                "LB" => reference.label = Some(value),
                "M1" => reference.number = Some(value),
                "M3" => reference.type_of_work = Some(value),
                "N1" => reference.notes.push(value),
                "N2" => reference.abstrct = Some(value),
                "NV" => reference.number_of_volumes = Some(value),
                "OP" => reference.original_publication = Some(value),
                "PB" => reference.publisher = Some(value),
                "PY" => reference.year = Some(value),
                "RI" => reference.reviewed_item = Some(value),
                "RN" => reference.research_notes = Some(value),
                "RP" => reference.reprint_edition = Some(value),
                "SE" => reference.section = Some(value),
                "SN" => reference.isbn_or_issn = Some(value),
                "SP" => reference.start_page = Some(value),
                "ST" => reference.short_title = Some(value),
                "T1" => reference.primary_title = Some(value),
                "T2" => reference.secondardy_title = Some(value),
                "T3" => reference.tertiary_title = Some(value),
                "TA" => reference.translated_authors.push(value),
                "TI" => reference.title = Some(value),
                "TT" => reference.translated_title = Some(value),
                "U1" => reference.type_of_work = Some(value),
                "UR" => reference.url = Some(value),
                "VL" => reference.volume = Some(value),
                "Y2" => reference.access_date = Some(value),
                _ => (),
            }
        }
        library
    }
}

impl From<RisReference> for Reference {
    fn from(reference: RisReference) -> Self {
        let csl_type = reference.ty.expect("RIS type is missing").csl();
        let mut date: FnvHashMap<DateVariable, DateOrRange> = FnvHashMap::default();
        let mut name: FnvHashMap<NameVariable, Vec<Name>> = FnvHashMap::default();
        let mut number: FnvHashMap<NumberVariable, NumberLike> = FnvHashMap::default();
        let mut ordinary: FnvHashMap<Variable, String> = FnvHashMap::default();

        if let Some(access_date) = reference.access_date {
            date.insert(DateVariable::Accessed, parse_date_or_range(access_date));
        }

        name.insert(NameVariable::Author, parse_authors(reference.authors));
        name.insert(NameVariable::Editor, parse_authors(reference.editors));

        if let Some(container_title) = reference
            .journal
            .or(reference.name_of_database)
            .or(reference.book_or_conference)
        {
            ordinary.insert(Variable::ContainerTitle, container_title.clone());
            ordinary.insert(Variable::CollectionTitle, container_title);
        }

        if let Some(value) = reference.date.or(reference.year) {
            let value = parse_date_or_range(value);
            date.insert(DateVariable::Issued, value.clone());
            date.insert(DateVariable::EventDate, value);
        }

        if let Some(url) = reference.url {
            ordinary.insert(Variable::URL, url);
        }

        let notes = reference.notes;
        ordinary.insert(
            Variable::Note,
            reference.research_notes.unwrap_or_else(|| notes.join("\n")),
        );

        if let Some(issue) = reference.issue {
            number.insert(NumberVariable::Issue, parse_number(issue));
        }

        if let Some(num) = reference.number {
            number.insert(NumberVariable::Number, parse_number(num));
        }

        if let Some(place) = reference.place {
            ordinary.insert(Variable::EventPlace, place.clone());
            ordinary.insert(Variable::PublisherPlace, place);
        }

        if let Some(abstrct) = reference.abstrct {
            ordinary.insert(Variable::Abstract, abstrct);
        }

        if let Some(call_number) = reference.call_number {
            ordinary.insert(Variable::CallNumber, call_number);
        }

        if let Some(doi) = reference.doi {
            ordinary.insert(Variable::DOI, doi);
        }

        if let Some(edition) = reference.edition {
            number.insert(NumberVariable::Edition, parse_number(edition));
        }

        if let Some(isbn_or_issn) = reference.isbn_or_issn {
            ordinary.insert(Variable::ISBN, isbn_or_issn.clone());
            ordinary.insert(Variable::ISSN, isbn_or_issn);
        }

        ordinary.insert(Variable::Keyword, reference.keywords.join(", "));

        if let Some(number_of_volumes) = reference.number_of_volumes {
            number.insert(
                NumberVariable::NumberOfVolumes,
                parse_number(number_of_volumes),
            );
        }

        if let Some(original_publication) = reference.original_publication {
            ordinary.insert(Variable::OriginalTitle, original_publication);
        }

        match (reference.start_page, reference.end_page) {
            (Some(start_page), Some(end_page)) => {
                number.insert(
                    NumberVariable::Page,
                    NumberLike::Str(format!("{}-{}", start_page, end_page)),
                );
            }
            (Some(page), None) | (None, Some(page)) => {
                number.insert(NumberVariable::Page, parse_number(page));
            }
            (None, None) => {}
        }

        if let Some(publisher) = reference.publisher {
            ordinary.insert(Variable::Publisher, publisher);
        }

        if let Some(reviewed_item) = reference.reviewed_item {
            ordinary.insert(Variable::ReviewedTitle, reviewed_item);
        }

        if let Some(section) = reference.section {
            ordinary.insert(Variable::Section, section);
        }

        if let Some(short_title) = reference.short_title {
            ordinary.insert(Variable::TitleShort, short_title);
        }

        if let Some(title) = reference.title {
            ordinary.insert(Variable::Title, title);
        }

        if let Some(volume) = reference.volume {
            number.insert(NumberVariable::Volume, parse_number(volume));
        }

        Reference {
            id: Atom::from(
                reference
                    .id
                    .or(reference.label)
                    .expect("RIS id is missing")
                    .as_str(),
            ),
            csl_type,
            language: None,
            name,
            number,
            date,
            ordinary,
        }
    }
}

fn parse_number(value: String) -> NumberLike {
    match value.parse() {
        Ok(value) => NumberLike::Num(value),
        Err(_) => NumberLike::Str(value),
    }
}

fn parse_authors(authors: Vec<String>) -> Vec<Name> {
    authors
        .into_iter()
        .map(|author| Name::Literal {
            is_latin_cyrillic: is_latin_cyrillic(&author),
            literal: author.into(),
        })
        .collect()
}

fn parse_date_or_range(value: String) -> DateOrRange {
    parse_date(&value)
        .map(DateOrRange::Single)
        .unwrap_or_else(|| DateOrRange::Literal {
            literal: value.into(),
            circa: false,
        })
}

fn parse_date(value: &str) -> Option<Date> {
    let mut parts = value.split('/');
    let year: i32 = parts.next()?.parse().ok()?;
    match parts.next().and_then(|p| p.parse().ok()) {
        Some(month) => match parts.next().and_then(|p| p.parse().ok()) {
            Some(day) => Some(Date::new(year, month, day)),
            None => Some(Date::new(year, month, 0)),
        },
        None => Some(Date::new(year, 0, 0)),
    }
}
