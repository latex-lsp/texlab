use std::fmt;

use syntax::bibtex::Value;

use super::{text::TextFieldData, FieldParseCache};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum NumberField {
    Edition,
    Number,
    Pages,
    PageTotal,
    Part,
    Volume,
    Volumes,
}

impl NumberField {
    pub fn parse(input: &str) -> Option<Self> {
        Some(match input.to_ascii_lowercase().as_str() {
            "edition" => Self::Edition,
            "number" => Self::Number,
            "pages" => Self::Pages,
            "pagetotal" => Self::PageTotal,
            "part" => Self::Part,
            "volume" => Self::Volume,
            "volumes" => Self::Volumes,
            _ => return None,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum NumberFieldData {
    Scalar(u32),
    Range(u32, u32),
    Other(String),
}

impl fmt::Display for NumberFieldData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(value) => write!(f, "{}", value),
            Self::Range(start, end) => write!(f, "{}-{}", start, end),
            Self::Other(value) => write!(f, "{}", value.replace("--", "-")),
        }
    }
}

impl NumberFieldData {
    pub fn parse(value: &Value, cache: &FieldParseCache) -> Option<Self> {
        let TextFieldData { text } = TextFieldData::parse(value, cache)?;
        text.split_once("--")
            .or_else(|| text.split_once('-'))
            .and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))
            .map(|(a, b)| Self::Range(a, b))
            .or_else(|| text.parse().ok().map(Self::Scalar))
            .or(Some(Self::Other(text)))
    }
}
