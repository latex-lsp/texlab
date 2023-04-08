pub mod bib;
pub mod log;
pub mod tex;

use rowan::TextRange;
use syntax::BuildError;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: TextRange,
    pub code: ErrorCode,
}

#[derive(Debug, Clone)]
pub enum ErrorCode {
    UnexpectedRCurly,
    RCurlyInserted,
    MismatchedEnvironment,
    ExpectingLCurly,
    ExpectingKey,
    ExpectingRCurly,
    ExpectingEq,
    ExpectingFieldValue,
    Build(BuildError),
}
