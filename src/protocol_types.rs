use lsp_types::{Location, LocationLink, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};
use serde_repr::*;

#[serde(untagged)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum DefinitionResponse {
    Locations(Vec<Location>),
    LocationLinks(Vec<LocationLink>),
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibtexFormattingOptions {
    pub line_length: Option<i32>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct LatexForwardSearchOptions {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum ForwardSearchStatus {
    Success = 0,
    Error = 1,
    Failure = 2,
    Unconfigured = 3,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ForwardSearchResult {
    pub status: ForwardSearchStatus,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LatexLintOptions {
    pub on_change: Option<bool>,
    pub on_save: Option<bool>,
}

impl LatexLintOptions {
    pub fn on_change(&self) -> bool {
        self.on_change.unwrap_or(false)
    }

    pub fn on_save(&self) -> bool {
        self.on_save.unwrap_or(false)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexBuildOptions {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
    pub on_save: Option<bool>,
}

impl LatexBuildOptions {
    pub fn executable(&self) -> String {
        self.executable
            .as_ref()
            .map(Clone::clone)
            .unwrap_or_else(|| "latexmk".to_owned())
    }

    pub fn args(&self) -> Vec<String> {
        self.args.as_ref().map(Clone::clone).unwrap_or_else(|| {
            vec![
                "-pdf".to_owned(),
                "-interaction=nonstopmode".to_owned(),
                "-synctex=1".to_owned(),
            ]
        })
    }

    pub fn on_save(&self) -> bool {
        self.on_save.unwrap_or(false)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum BuildStatus {
    Success = 0,
    Error = 1,
    Failure = 2,
    Cancelled = 3,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResult {
    pub status: BuildStatus,
}
