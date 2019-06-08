use super::*;
use std::path::Path;

pub const DATABASE_PATH: &str = "../tlpkg/texlive.tlpdb";

pub fn read_database(file: &Path, root_dir: &Path) -> Option<Vec<PackageManifest>> {
    unimplemented!()
}
