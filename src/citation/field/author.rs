use std::{borrow::Cow, fmt, str::FromStr};

use human_name::Name;
use itertools::Itertools;
use strum::EnumString;

use crate::syntax::bibtex::Field;

use super::text::TextFieldData;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum AuthorField {
    Afterword,
    Annotator,
    Author,
    Commentator,
    Editor,
    EditorA,
    EditorB,
    EditorC,
    Foreword,
    Introduction,
    Translator,
}

impl AuthorField {
    pub fn parse(input: &str) -> Option<Self> {
        Self::from_str(input).ok()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AuthorFieldData {
    pub authors: Vec<Name>,
}

impl fmt::Display for AuthorFieldData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self
            .authors
            .iter()
            .map(|author| author.display_initial_surname());

        for part in Itertools::intersperse(names, Cow::Borrowed(", ")) {
            write!(f, "{}", part)?;
        }

        Ok(())
    }
}

impl AuthorFieldData {
    pub fn parse(field: &Field) -> Option<Self> {
        let TextFieldData { text } = TextFieldData::parse(field)?;
        let mut authors = Vec::new();
        let mut words = Vec::new();
        for word in text.split_whitespace() {
            if word.eq_ignore_ascii_case("and") {
                authors.push(Name::parse(&words.join(" "))?);
                words.clear();
            } else {
                words.push(word);
            }
        }

        if !words.is_empty() {
            authors.push(Name::parse(&words.join(" "))?);
        }

        Some(Self { authors })
    }
}
