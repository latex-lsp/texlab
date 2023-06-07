use commands::ForwardSearchError;
use lsp_types::{Position, Range, TextDocumentIdentifier, TextDocumentPositionParams};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub struct BuildRequest;

impl lsp_types::request::Request for BuildRequest {
    type Params = BuildParams;

    type Result = BuildResult;

    const METHOD: &'static str = "textDocument/build";
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildParams {
    pub text_document: TextDocumentIdentifier,

    #[serde(default)]
    pub position: Option<Position>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResult {
    pub status: BuildStatus,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum BuildStatus {
    SUCCESS = 0,
    ERROR = 1,
    FAILURE = 2,
    CANCELLED = 3,
}

pub struct ForwardSearchRequest;

impl lsp_types::request::Request for ForwardSearchRequest {
    type Params = TextDocumentPositionParams;

    type Result = ForwardSearchResult;

    const METHOD: &'static str = "textDocument/forwardSearch";
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum ForwardSearchStatus {
    SUCCESS = 0,
    ERROR = 1,
    FAILURE = 2,
    UNCONFIGURED = 3,
}

impl From<ForwardSearchError> for ForwardSearchStatus {
    fn from(why: ForwardSearchError) -> Self {
        match why {
            ForwardSearchError::Unconfigured => ForwardSearchStatus::UNCONFIGURED,
            ForwardSearchError::NotLocal(_) => ForwardSearchStatus::FAILURE,
            ForwardSearchError::InvalidPath(_) => ForwardSearchStatus::ERROR,
            ForwardSearchError::TexNotFound(_) => ForwardSearchStatus::FAILURE,
            ForwardSearchError::PdfNotFound(_) => ForwardSearchStatus::ERROR,
            ForwardSearchError::LaunchViewer(_) => ForwardSearchStatus::ERROR,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForwardSearchResult {
    pub status: ForwardSearchStatus,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentLocation {
    pub name: TextWithRange,
    pub full_range: Range,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextWithRange {
    pub text: String,
    pub range: Range,
}
