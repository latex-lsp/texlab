use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
};

use anyhow::Result;
use rustc_hash::FxHashSet;

use crate::db::Language;

#[derive(Debug)]
pub struct DistroFile(PathBuf);

impl DistroFile {
    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn name(&self) -> &str {
        self.0.file_name().unwrap().to_str().unwrap()
    }
}

impl PartialEq for DistroFile {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for DistroFile {}

impl std::hash::Hash for DistroFile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state)
    }
}

impl Borrow<str> for DistroFile {
    fn borrow(&self) -> &str {
        self.name()
    }
}

#[derive(Debug, Default)]
pub struct FileNameDB {
    files: FxHashSet<DistroFile>,
}

impl FileNameDB {
    pub fn get(&self, name: &str) -> Option<&Path> {
        self.files.get(name).map(|file| file.path())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Path)> + '_ {
        self.files.iter().map(|file| (file.name(), file.path()))
    }

    pub fn parse(
        root_dirs: &[PathBuf],
        reader: &mut dyn FnMut(&Path) -> Result<Vec<PathBuf>>,
    ) -> Result<Self> {
        let files = root_dirs
            .iter()
            .flat_map(|dir| reader(dir))
            .flatten()
            .filter_map(|rel_path| {
                Language::from_path(&rel_path)?;
                rel_path.file_name()?.to_str()?;
                let abs_path = root_dirs
                    .iter()
                    .rev()
                    .map(|dir| dir.join(&rel_path))
                    .find_map(|path| std::fs::canonicalize(path).ok())?;
                Some(DistroFile(abs_path))
            })
            .collect();

        Ok(Self { files })
    }
}
