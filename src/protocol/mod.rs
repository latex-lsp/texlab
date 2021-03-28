use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub use self::{capabilities::ClientCapabilitiesExt, options::*, range::RangeExt, uri::Uri};

mod capabilities;
mod options;
mod range;
mod uri;

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
