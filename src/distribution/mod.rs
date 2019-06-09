use std::path::{Path, PathBuf};
use std::process::Command;

mod ini;
mod miktex;
mod texlive;
mod tlpdb;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct PackageManifest {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub doc_files: Vec<PathBuf>,
    pub run_files: Vec<PathBuf>,
    pub is_installed: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TexDistributionKind {
    Texlive,
    Miktex,
}

impl Default for TexDistributionKind {
    fn default() -> Self {
        TexDistributionKind::Texlive
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct TexDistribution {
    pub kind: TexDistributionKind,
    pub packages: Vec<PackageManifest>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    KpsewhichNotFound,
    UnsupportedTexDistribution,
    CorruptPackageDatabase,
}

pub type Result<T> = std::result::Result<T, Error>;

impl TexDistribution {
    pub fn load() -> Result<Self> {
        let root_dir = Self::query_path_variable("TEXMFDIST")?;
        let (file, kind) = Self::find_database(&root_dir)?;
        let packages = match kind {
            TexDistributionKind::Texlive => texlive::read_database(&file, &root_dir),
            TexDistributionKind::Miktex => miktex::read_database(&file, &root_dir),
        }
        .ok_or(Error::CorruptPackageDatabase)?;

        Ok(Self { kind, packages })
    }

    fn query_path_variable(var: &str) -> Result<PathBuf> {
        let output = Command::new("kpsewhich")
            .args(&["-var-value", var])
            .output()
            .map_err(|_| Error::KpsewhichNotFound)?;

        let path = PathBuf::from(
            String::from_utf8(output.stdout)
                .expect("Could not decode output from kpsewhich")
                .lines()
                .nth(0)
                .expect("Invalid output from kpsewhich")
                .to_owned(),
        );

        Ok(path)
    }

    fn find_database(directory: &Path) -> Result<(PathBuf, TexDistributionKind)> {
        let texlive_file = directory.join(texlive::DATABASE_PATH);
        if texlive_file.is_file() {
            return Ok((texlive_file, TexDistributionKind::Texlive));
        }

        let miktex_file = directory.join(miktex::DATABASE_PATH);
        if miktex_file.is_file() {
            return Ok((miktex_file, TexDistributionKind::Miktex));
        }

        Err(Error::UnsupportedTexDistribution)
    }
}
