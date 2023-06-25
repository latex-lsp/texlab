use rowan::TextRange;
use syntax::BuildError;

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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BibError {
    ExpectingLCurly,
    ExpectingKey,
    ExpectingRCurly,
    ExpectingEq,
    ExpectingFieldValue,
    UnusedEntry,
}
