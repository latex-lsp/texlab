mod capabilities;
mod client;
mod codec;
mod options;
mod range;

pub use self::capabilities::ClientCapabilitiesExt;
pub use self::client::{LatexLspClient, LspClient};
pub use self::codec::LspCodec;
pub use self::options::*;
pub use self::range::RangeExt;

use lsp_types::{Location, LocationLink, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};
use serde_repr::*;

#[serde(untagged)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum DefinitionResponse {
    Locations(Vec<Location>),
    LocationLinks(Vec<LocationLink>),
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildParams {
    pub text_document: TextDocumentIdentifier,
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
