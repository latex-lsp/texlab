use super::Language;
use futures::Future;
use std::{
    collections::HashMap,
    env, error,
    ffi::OsStr,
    fmt, io,
    path::{Path, PathBuf},
    string::FromUtf8Error,
};
use tokio::fs;
use tokio::process::Command;

#[derive(Debug)]
pub enum KpsewhichError {
    IO(io::Error),
    Decode(FromUtf8Error),
    InvalidOutput,
    NotInstalled,
    NoDatabase,
    CorruptDatabase,
}

impl From<io::Error> for KpsewhichError {
    fn from(why: io::Error) -> Self {
        Self::IO(why)
    }
}

impl From<FromUtf8Error> for KpsewhichError {
    fn from(why: FromUtf8Error) -> Self {
        Self::Decode(why)
    }
}

impl fmt::Display for KpsewhichError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(why) => write!(f, "{}", why),
            Self::Decode(why) => write!(f, "{}", why),
            Self::InvalidOutput => write!(f, "invalid output from kpsewhich"),
            Self::NotInstalled => write!(f, "kpsewhich not installed"),
            Self::NoDatabase => write!(f, "no kpsewhich database"),
            Self::CorruptDatabase => write!(f, "corrupt kpsewhich database"),
        }
    }
}

impl error::Error for KpsewhichError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IO(why) => why.source(),
            Self::Decode(why) => why.source(),
            Self::InvalidOutput | Self::NotInstalled | Self::NoDatabase | Self::CorruptDatabase => {
                None
            }
        }
    }
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
