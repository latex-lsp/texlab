use super::Language;
use futures::Future;
use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    string::FromUtf8Error,
};
use thiserror::Error;
use tokio::{fs, process::Command};

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
    #[error("no kpsewhich database")]
    NoDatabase,
    #[error("corrupt kpsewhich database")]
    CorruptDatabase,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Resolver {
    pub files_by_name: HashMap<String, PathBuf>,
}

impl Resolver {
    pub fn new(files_by_name: HashMap<String, PathBuf>) -> Self {
        Self { files_by_name }
    }
}

pub async fn parse_database<'a, R, F>(
    root_directories: &'a [PathBuf],
    reader: R,
) -> Result<Resolver, KpsewhichError>
where
    R: Fn(&'a Path) -> F,
    F: Future<Output = Result<Vec<PathBuf>, KpsewhichError>>,
{
    let mut files_by_name = HashMap::new();
    for directory in root_directories {
        for path in reader(directory).await? {
            if is_tex_file(&path) {
                if let Some(path) = make_absolute(root_directories, &path).await {
                    if let Some(name) = path
                        .file_name()
                        .and_then(OsStr::to_str)
                        .map(ToString::to_string)
                    {
                        files_by_name.insert(name, path);
                    }
                }
            }
        }
    }
    Ok(Resolver::new(files_by_name))
}

fn is_tex_file(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .and_then(Language::by_extension)
        .is_some()
}

async fn make_absolute(root_directories: &[PathBuf], relative_path: &Path) -> Option<PathBuf> {
    for dir in root_directories.iter().rev() {
        if let Ok(path) = fs::canonicalize(dir.join(&relative_path)).await {
            return Some(path);
        }
    }
    None
}

pub async fn root_directories() -> Result<Vec<PathBuf>, KpsewhichError> {
    let texmf = run(&["-var-value", "TEXMF"]).await?;
    let expand_arg = format!("--expand-braces={}", texmf);
    let expanded = run(&[&expand_arg]).await?;
    let directories = env::split_paths(&expanded.replace("!", ""))
        .filter(|path| path.exists())
        .collect();
    Ok(directories)
}

async fn run<I, S>(args: I) -> Result<String, KpsewhichError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("kpsewhich")
        .args(args)
        .output()
        .await
        .map_err(|_| KpsewhichError::NotInstalled)?;

    let result = String::from_utf8(output.stdout)?
        .lines()
        .next()
        .ok_or(KpsewhichError::InvalidOutput)?
        .into();

    Ok(result)
}
