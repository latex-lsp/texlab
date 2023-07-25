use rowan::TextRange;
use syntax::BuildError;
use url::Url;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Diagnostic {
    pub range: TextRange,
    pub data: DiagnosticData,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DiagnosticData {
    Tex(TexError),
    Bib(BibError),
    Build(BuildError),
}

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
