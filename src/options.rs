use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub root_directory: Option<PathBuf>,
    pub aux_directory: Option<PathBuf>,
}
