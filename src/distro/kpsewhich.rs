use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

use crate::DocumentLanguage;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Resolver {
    pub files_by_name: FxHashMap<SmolStr, PathBuf>,
}

impl Resolver {
    pub fn new(files_by_name: FxHashMap<SmolStr, PathBuf>) -> Self {
        Self { files_by_name }
    }
}

pub fn parse_database(
    root_directories: &[PathBuf],
    mut reader: impl FnMut(&Path) -> Result<Vec<PathBuf>>,
) -> Result<Resolver> {
    let mut files_by_name = FxHashMap::default();
    for directory in root_directories {
        for path in reader(directory)? {
            if DocumentLanguage::by_path(&path).is_some() {
                if let Some(path) = make_absolute(root_directories, &path) {
                    if let Some(name) = path.file_name().and_then(OsStr::to_str).map(Into::into) {
                        files_by_name.insert(name, path);
                    }
                }
            }
        }
    }
    Ok(Resolver::new(files_by_name))
}

fn make_absolute(root_directories: &[PathBuf], relative_path: &Path) -> Option<PathBuf> {
    for dir in root_directories.iter().rev() {
        if let Ok(path) = fs::canonicalize(dir.join(&relative_path)) {
            return Some(path);
        }
    }
    None
}

pub fn root_directories() -> Result<Vec<PathBuf>> {
    let texmf = run(&["-var-value", "TEXMF"])?;
    let expand_arg = format!("--expand-braces={}", texmf);
    let expanded = run(&[&expand_arg])?;
    let directories = env::split_paths(&expanded.replace('!', ""))
        .filter(|path| path.exists())
        .collect();
    Ok(directories)
}

fn run(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<String> {
    let output = Command::new("kpsewhich").args(args).output()?;

    let result = String::from_utf8(output.stdout)?
        .lines()
        .next()
        .unwrap()
        .into();

    Ok(result)
}
