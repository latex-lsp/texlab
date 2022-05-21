use std::{fmt, str::FromStr};

use strum::EnumString;

use crate::syntax::bibtex::Field;

use super::text::TextFieldData;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, EnumString)]
#[strum(ascii_case_insensitive)]
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
        Self::from_str(input).ok()
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
            Self::Other(value) => write!(f, "{}", value),
        }
    }
}

impl NumberFieldData {
    pub fn parse(field: &Field) -> Option<Self> {
        let TextFieldData { text } = TextFieldData::parse(field)?;
        text.split_once("--")
            .or_else(|| text.split_once('-'))
            .and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))
            .map(|(a, b)| Self::Range(a, b))
            .or_else(|| text.parse().ok().map(Self::Scalar))
            .or(Some(Self::Other(text)))
    }
}
