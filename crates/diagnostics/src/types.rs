use rowan::TextRange;
use syntax::BuildError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Diagnostic {
    pub range: TextRange,
    pub data: DiagnosticData,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DiagnosticData {
    Syntax(SyntaxError),
    Build(BuildError),
    Label(LabelError),
    Citation(CitationError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SyntaxError {
    UnexpectedRCurly,
    RCurlyInserted,
    MismatchedEnvironment,
    ExpectingLCurly,
    ExpectingKey,
    ExpectingRCurly,
    ExpectingEq,
    ExpectingFieldValue,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LabelError {
    Unused,
    Undefined,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CitationError {
    Unused,
    Undefined,
}
