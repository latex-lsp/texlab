use std::str::FromStr;

use rustc_hash::FxHashMap;
use strum::EnumString;

use crate::syntax::bibtex::{Entry, Field, HasName, HasType};

use super::field::{
    author::{AuthorField, AuthorFieldData},
    date::{DateField, DateFieldData},
    number::{NumberField, NumberFieldData},
    text::{TextField, TextFieldData},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, EnumString)]
#[strum(ascii_case_insensitive)]
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
    pub fn parse(entry: &Entry) -> Option<Self> {
        let mut data = EntryData {
            kind: entry
                .type_token()
                .and_then(|token| EntryKind::from_str(&token.text()[1..]).ok())
                .unwrap_or(EntryKind::Unknown),
            ..EntryData::default()
        };

        for field in entry.fields() {
            let _ = data.parse_field(&field);
        }

        Some(data)
    }

    fn parse_field(&mut self, field: &Field) -> Option<()> {
        let name = field.name_token()?;
        let name = name.text();
        self.parse_author_field(field, name)
            .or_else(|| self.parse_date_field(field, name))
            .or_else(|| self.parse_number_field(field, name))
            .or_else(|| self.parse_text_field(field, name))
    }

    fn parse_author_field(&mut self, field: &Field, name: &str) -> Option<()> {
        let name = AuthorField::parse(name)?;
        let data = AuthorFieldData::parse(field)?;
        self.author.insert(name, data);
        Some(())
    }

    fn parse_date_field(&mut self, field: &Field, name: &str) -> Option<()> {
        let name = DateField::parse(name)?;
        let data = DateFieldData::parse(field)?;
        self.date.insert(name, data);
        Some(())
    }

    fn parse_number_field(&mut self, field: &Field, name: &str) -> Option<()> {
        let name = NumberField::parse(name)?;
        let data = NumberFieldData::parse(field)?;
        self.number.insert(name, data);
        Some(())
    }

    fn parse_text_field(&mut self, field: &Field, name: &str) -> Option<()> {
        let name = TextField::parse(name).unwrap_or(TextField::Unknown);
        let data = TextFieldData::parse(field)?;
        self.text.insert(name, data);
        Some(())
    }
}
