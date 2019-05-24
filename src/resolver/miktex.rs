use super::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub const DATABASE_PATH: &'static str = "miktex/data/le";

pub fn read_database(directory: &Path, root_directories: &[PathBuf]) -> Result<Vec<PathBuf>> {
    unimplemented!()
}
