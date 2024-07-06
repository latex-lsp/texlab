use std::path::PathBuf;

use rustc_hash::FxHashSet;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct FileList {
    pub working_dir: Option<PathBuf>,
    pub inputs: FxHashSet<PathBuf>,
    pub outputs: FxHashSet<PathBuf>,
}
