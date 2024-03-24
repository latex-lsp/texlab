use base_db::semantics::bib::Semantics;
use bibtex_utils::field::{
    author::{AuthorField, AuthorFieldData},
    date::{DateField, DateFieldData},
    number::{NumberField, NumberFieldData},
    text::{TextField, TextFieldData},
    FieldParseCache,
};
use rustc_hash::FxHashMap;
use syntax::bibtex::{Entry, Field, HasName, HasType, HasValue, Value};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum EntryKind {
    Article,
    Book,
    MVBook,
    InBook,
    BookInBook,
    SuppBook,
    Booklet,
    Collection,
    MVCollection,
    InCollection,
    SuppCollection,
    DataSet,
    Manual,
    Misc,
    Online,
    Electronic,
    Www,
    Patent,
    Periodical,
    SuppPeriodical,
    Proceedings,
    MVProceedings,
    InProceedings,
    Conference,
    Reference,
    MVReference,
    InReference,
    Report,
    Set,
    Software,
    Thesis,
    MasterThesis,
    PhdThesis,
    TechReport,
    Unknown,
}

impl EntryKind {
    pub fn parse(input: &str) -> Self {
        match input.to_ascii_lowercase().as_str() {
            "article" => Self::Article,
            "book" => Self::Book,
            "mvbook" => Self::MVBook,
            "inbook" => Self::InBook,
            "bookinbook" => Self::BookInBook,
            "suppbook" => Self::SuppBook,
            "booklet" => Self::Booklet,
            "collection" => Self::Collection,
            "mvcollection" => Self::MVCollection,
            "incollection" => Self::InCollection,
            "suppcollection" => Self::SuppCollection,
            "dataset" => Self::DataSet,
            "manual" => Self::Manual,
            "misc" => Self::Misc,
            "online" => Self::Online,
            "electronic" => Self::Electronic,
            "www" => Self::Www,
            "patent" => Self::Patent,
            "periodical" => Self::Periodical,
            "suppperiodical" => Self::SuppPeriodical,
            "proceedings" => Self::Proceedings,
            "mvproceedings" => Self::MVProceedings,
            "inproceedings" => Self::InProceedings,
            "conference" => Self::Conference,
            "reference" => Self::Reference,
            "mvreference" => Self::MVReference,
            "inreference" => Self::InReference,
            "report" => Self::Report,
            "set" => Self::Set,
            "software" => Self::Software,
            "thesis" => Self::Thesis,
            "masterthesis" => Self::MasterThesis,
            "phdthesis" => Self::PhdThesis,
            "techreport" => Self::TechReport,
            _ => Self::Unknown,
        }
    }
}

impl Default for EntryKind {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntryData {
    pub kind: EntryKind,
    pub text: FxHashMap<TextField, TextFieldData>,
    pub author: FxHashMap<AuthorField, AuthorFieldData>,
    pub date: FxHashMap<DateField, DateFieldData>,
    pub number: FxHashMap<NumberField, NumberFieldData>,
}

impl EntryData {
    pub fn from_entry(entry: &Entry, semantics: &Semantics) -> Self {
        let mut data = EntryData {
            kind: entry.type_token().map_or(EntryKind::Unknown, |token| {
                EntryKind::parse(&token.text()[1..])
            }),
            ..EntryData::default()
        };

        for field in entry.fields() {
            let _ = data.parse_field(&field, &semantics.expanded_defs);
        }

        data
    }
}

impl EntryData {
    fn parse_field(&mut self, field: &Field, expanded_defs: &FieldParseCache) -> Option<()> {
        let name = field.name_token()?;
        let name = name.text();
        let value = field.value()?;
        self.parse_author_field(name, &value, expanded_defs)
            .or_else(|| self.parse_date_field(name, &value, expanded_defs))
            .or_else(|| self.parse_number_field(name, &value, expanded_defs))
            .or_else(|| self.parse_text_field(name, &value, expanded_defs))
    }

    fn parse_author_field(
        &mut self,
        name: &str,
        value: &Value,
        expanded_defs: &FieldParseCache,
    ) -> Option<()> {
        let name = AuthorField::parse(name)?;
        let data = AuthorFieldData::parse(value, expanded_defs)?;
        self.author.insert(name, data);
        Some(())
    }

    fn parse_date_field(
        &mut self,
        name: &str,
        value: &Value,
        expanded_defs: &FieldParseCache,
    ) -> Option<()> {
        let name = DateField::parse(name)?;
        let data = DateFieldData::parse(value, expanded_defs)?;
        self.date.insert(name, data);
        Some(())
    }

    fn parse_number_field(
        &mut self,
        name: &str,
        value: &Value,
        expanded_defs: &FieldParseCache,
    ) -> Option<()> {
        let name = NumberField::parse(name)?;
        let data = NumberFieldData::parse(value, expanded_defs)?;
        self.number.insert(name, data);
        Some(())
    }

    fn parse_text_field(
        &mut self,
        name: &str,
        value: &Value,
        expanded_defs: &FieldParseCache,
    ) -> Option<()> {
        let name = TextField::parse(name).unwrap_or(TextField::Unknown);
        let data = TextFieldData::parse(value, expanded_defs)?;
        self.text.insert(name, data);
        Some(())
    }
}
