use line_index::LineCol;
use rowan::TextRange;
use syntax::BuildError;
use url::Url;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TexError {
    UnexpectedRCurly,
    ExpectingRCurly,
    MismatchedEnvironment,
    UnusedLabel,
    UndefinedLabel,
    UndefinedCitation,
    DuplicateLabel(Vec<(Url, TextRange)>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BibError {
    ExpectingLCurly,
    ExpectingKey,
    ExpectingRCurly,
    ExpectingEq,
    ExpectingFieldValue,
    UnusedEntry,
    DuplicateEntry(Vec<(Url, TextRange)>),
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
}
