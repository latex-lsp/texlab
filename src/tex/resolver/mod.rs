use crate::syntax::Language;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

mod miktex;
mod texlive;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    KpsewhichNotFound,
    UnsupportedTexDistribution,
    CorruptFileDatabase,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TexDistributionKind {
    Texlive,
    Miktex,
}

lazy_static! {
    pub static ref TEX_RESOLVER: Result<Arc<TexResolver>> = TexResolver::load().map(Arc::new);
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TexResolver {
    pub files_by_name: HashMap<OsString, PathBuf>,
}

impl TexResolver {
    fn load() -> Result<Self> {
        let root_directories = Self::find_root_directories()?;
        let kind = Self::detect_distribution(&root_directories)?;
        let reader = match kind {
            TexDistributionKind::Texlive => texlive::read_database,
            TexDistributionKind::Miktex => miktex::read_database,
        };

        let mut files_by_name = HashMap::new();
        for directory in &root_directories {
            let database = reader(directory)?
                .into_iter()
                .filter(|path| {
                    path.extension()
                        .and_then(OsStr::to_str)
                        .and_then(Language::by_extension)
                        .is_some()
                })
                .filter_map(|path| {
                    root_directories
                        .iter()
                        .rev()
                        .find_map(move |dir| dir.join(&path).canonicalize().ok())
                })
                .map(|path| (path.file_name().unwrap().to_owned(), path));

            files_by_name.extend(database);
        }

        Ok(Self { files_by_name })
    }

    fn find_root_directories() -> Result<Vec<PathBuf>> {
        let texmf = Self::run_kpsewhich(&["-var-value", "TEXMF"])?;
        let expanded = Self::run_kpsewhich(&[&format!("--expand-braces={}", texmf)])?;
        let directories = env::split_paths(&expanded.replace("!", ""))
            .filter(|x| x.exists())
            .collect();
        Ok(directories)
    }

    fn run_kpsewhich(args: &[&str]) -> Result<String> {
        let output = Command::new("kpsewhich")
            .args(args)
            .output()
            .map_err(|_| Error::KpsewhichNotFound)?;
        Ok(String::from_utf8(output.stdout)
            .expect("Could not decode output from kpsewhich")
            .lines()
            .next()
            .expect("Invalid output from kpsewhich")
            .to_owned())
    }

    fn detect_distribution(directories: &[PathBuf]) -> Result<TexDistributionKind> {
        for directory in directories {
            if directory.join(texlive::DATABASE_PATH).exists() {
                return Ok(TexDistributionKind::Texlive);
            } else if directory.join(miktex::DATABASE_PATH).exists() {
                return Ok(TexDistributionKind::Miktex);
            }
        }

        Err(Error::UnsupportedTexDistribution)
    }
}
