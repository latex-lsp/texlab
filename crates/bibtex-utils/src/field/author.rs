use std::{borrow::Cow, fmt};

use human_name::Name;
use itertools::Itertools;
use syntax::bibtex::Value;

use super::{text::TextFieldData, FieldParseCache};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
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
        Some(match input.to_ascii_lowercase().as_str() {
            "afterword" => Self::Afterword,
            "annotator" => Self::Annotator,
            "author" => Self::Author,
            "commentator" => Self::Commentator,
            "editor" => Self::Editor,
            "editora" => Self::EditorA,
            "editorb" => Self::EditorB,
            "editorc" => Self::EditorC,
            "foreword" => Self::Foreword,
            "introduction" => Self::Introduction,
            "translator" => Self::Translator,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct AuthorFieldData {
    pub authors: Vec<Name>,
}

impl fmt::Display for AuthorFieldData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self.authors.iter().map(Name::display_initial_surname);

        for part in Itertools::intersperse(names, Cow::Borrowed(", ")) {
            write!(f, "{}", part)?;
        }

        Ok(())
    }
}

impl AuthorFieldData {
    pub fn parse(value: &Value, cache: &FieldParseCache) -> Option<Self> {
        let TextFieldData { text } = TextFieldData::parse(value, cache)?;
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
