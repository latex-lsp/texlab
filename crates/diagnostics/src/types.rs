use line_index::{LineCol, LineIndex};
use rowan::TextRange;
use syntax::BuildError;
use url::Url;

#[derive(PartialEq, Eq, Clone)]
pub enum TexError {
    UnexpectedRCurly,
    ExpectingRCurly,
    MismatchedEnvironment,
    UnusedLabel,
    UndefinedLabel,
    UndefinedCitation,
    DuplicateLabel(Vec<(Url, TextRange)>),
}

impl std::fmt::Debug for TexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedRCurly => write!(f, "UnexpectedRCurly"),
            Self::ExpectingRCurly => write!(f, "ExpectingRCurly"),
            Self::MismatchedEnvironment => write!(f, "MismatchedEnvironment"),
            Self::UnusedLabel => write!(f, "UnusedLabel"),
            Self::UndefinedLabel => write!(f, "UndefinedLabel"),
            Self::UndefinedCitation => write!(f, "UndefinedCitation"),
            Self::DuplicateLabel(locations) => {
                let mut t = f.debug_tuple("DuplicateLabel");
                for (uri, range) in locations {
                    t.field(&(uri.as_str(), range));
                }

                t.finish()
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum BibError {
    ExpectingLCurly,
    ExpectingKey,
    ExpectingRCurly,
    ExpectingEq,
    ExpectingFieldValue,
    UnusedEntry,
    DuplicateEntry(Vec<(Url, TextRange)>),
}

impl std::fmt::Debug for BibError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectingLCurly => write!(f, "ExpectingLCurly"),
            Self::ExpectingKey => write!(f, "ExpectingKey"),
            Self::ExpectingRCurly => write!(f, "ExpectingRCurly"),
            Self::ExpectingEq => write!(f, "ExpectingEq"),
            Self::ExpectingFieldValue => write!(f, "ExpectingFieldValue"),
            Self::UnusedEntry => write!(f, "UnusedEntry"),
            Self::DuplicateEntry(locations) => {
                let mut t = f.debug_tuple("DuplicateEntry");
                for (uri, range) in locations {
                    t.field(&(uri.as_str(), range));
                }

                t.finish()
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ChktexError {
    pub start: LineCol,
    pub end: LineCol,
    pub message: String,
    pub severity: ChktexSeverity,
    pub code: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ChktexSeverity {
    Error,
    Warning,
    Message,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Diagnostic {
    Tex(TextRange, TexError),
    Bib(TextRange, BibError),
    Build(TextRange, BuildError),
    Chktex(ChktexError),
}

impl Diagnostic {
    pub fn message(&self) -> &str {
        match self {
            Diagnostic::Tex(_, error) => match error {
                TexError::UnexpectedRCurly => "Unexpected \"}\"",
                TexError::ExpectingRCurly => "Expecting a curly bracket: \"}\"",
                TexError::MismatchedEnvironment => "Mismatched environment",
                TexError::UnusedLabel => "Unused label",
                TexError::UndefinedLabel => "Undefined reference",
                TexError::UndefinedCitation => "Undefined reference",
                TexError::DuplicateLabel(_) => "Duplicate label",
            },
            Diagnostic::Bib(_, error) => match error {
                BibError::ExpectingLCurly => "Expecting a curly bracket: \"{\"",
                BibError::ExpectingKey => "Expecting a key",
                BibError::ExpectingRCurly => "Expecting a curly bracket: \"}\"",
                BibError::ExpectingEq => "Expecting an equality sign: \"=\"",
                BibError::ExpectingFieldValue => "Expecting a field value",
                BibError::UnusedEntry => "Unused entry",
                BibError::DuplicateEntry(_) => "Duplicate entry key",
            },
            Diagnostic::Build(_, error) => &error.message,
            Diagnostic::Chktex(error) => &error.message,
        }
    }

    pub fn range(&self, line_index: &LineIndex) -> Option<TextRange> {
        Some(match self {
            Diagnostic::Tex(range, _) => *range,
            Diagnostic::Bib(range, _) => *range,
            Diagnostic::Build(range, _) => *range,
            Diagnostic::Chktex(error) => {
                let start = line_index.offset(error.start)?;
                let end = line_index.offset(error.end)?;
                TextRange::new(start, end)
            }
        })
    }

    pub fn additional_locations_mut(&mut self) -> Option<&mut Vec<(Url, TextRange)>> {
        match self {
            Diagnostic::Tex(_, err) => match err {
                TexError::UnexpectedRCurly
                | TexError::ExpectingRCurly
                | TexError::MismatchedEnvironment
                | TexError::UnusedLabel
                | TexError::UndefinedLabel
                | TexError::UndefinedCitation => None,
                TexError::DuplicateLabel(locations) => Some(locations),
            },
            Diagnostic::Bib(_, err) => match err {
                BibError::ExpectingLCurly
                | BibError::ExpectingKey
                | BibError::ExpectingRCurly
                | BibError::ExpectingEq
                | BibError::ExpectingFieldValue
                | BibError::UnusedEntry => None,
                BibError::DuplicateEntry(locations) => Some(locations),
            },
            Diagnostic::Chktex(_) => None,
            Diagnostic::Build(_, _) => None,
        }
    }
}
