use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

mod miktex;
mod texlive;

#[derive(Debug)]
pub enum Error {
    KpsewhichNotFound,
    UnsupportedTexDistribution,
    CorruptFileDatabase,
}

pub type Result<T> = std::result::Result<T, Error>;

pub enum TexDistributionKind {
    Texlive,
    Miktex,
}

#[derive(Debug)]
pub struct TexResolver {
    pub files_by_name: HashMap<OsString, PathBuf>,
}

impl TexResolver {
    pub fn new() -> Self {
        Self {
            files_by_name: HashMap::new(),
        }
    }

    pub fn load() -> Result<Self> {
        let directories = Self::find_root_directories()?;
        let kind = Self::detect_distribution(&directories)?;
        let files_by_name = Self::read_database(&directories, kind)?;
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
            .nth(0)
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

    fn read_database(
        root_directories: &[PathBuf],
        kind: TexDistributionKind,
    ) -> Result<HashMap<OsString, PathBuf>> {
        let mut files_by_name = HashMap::new();
        for directory in root_directories {
            let database = match kind {
                TexDistributionKind::Texlive => texlive::read_database(&directory),
                TexDistributionKind::Miktex => miktex::read_database(&directory, root_directories),
            }?;

            for file in database {
                let name = file.file_name().unwrap().to_owned();
                files_by_name.insert(name, file);
            }
        }

        Ok(files_by_name)
    }
}
