use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    string::FromUtf8Error,
};

use rustc_hash::FxHashMap;
use smol_str::SmolStr;
use thiserror::Error;

use crate::DocumentLanguage;

#[derive(Debug, Error)]
pub enum KpsewhichError {
    #[error("an I/O error occurred: `{0}`")]
    IO(#[from] io::Error),

    #[error("an utf8 error occurred: `{0}`")]
    Decode(#[from] FromUtf8Error),

    #[error("invalid output from kpsewhich")]
    InvalidOutput,

    #[error("kpsewhich not installed")]
    NotInstalled,

    #[error("no kpsewhich db")]
    NoDatabase,

    #[error("corrupt kpsewhich db")]
    CorruptDatabase,
}

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
    mut reader: impl FnMut(&Path) -> Result<Vec<PathBuf>, KpsewhichError>,
) -> Result<Resolver, KpsewhichError> {
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

pub fn root_directories() -> Result<Vec<PathBuf>, KpsewhichError> {
    let texmf = run(&["-var-value", "TEXMF"])?;
    let expand_arg = format!("--expand-braces={}", texmf);
    let expanded = run(&[&expand_arg])?;
    let directories = env::split_paths(&expanded.replace("!", ""))
        .filter(|path| path.exists())
        .collect();
    Ok(directories)
}

fn run(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<String, KpsewhichError> {
    let output = Command::new("kpsewhich")
        .args(args)
        .output()
        .map_err(|_| KpsewhichError::NotInstalled)?;

    let result = String::from_utf8(output.stdout)?
        .lines()
        .next()
        .ok_or(KpsewhichError::InvalidOutput)?
        .into();

    Ok(result)
}
