mod file_name_db;
mod kpsewhich;
mod miktex;
mod texlive;

use std::process::{Command, Stdio};

use anyhow::Result;

pub use file_name_db::FileNameDB;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DistroKind {
    Texlive,
    Miktex,
    Tectonic,
    Unknown,
}

impl Default for DistroKind {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Default)]
pub struct Distro {
    pub kind: DistroKind,
    pub file_name_db: FileNameDB,
}

impl Distro {
    pub fn detect() -> Result<Self> {
        let kind = match Command::new("latex").arg("--version").output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("TeX Live") {
                    DistroKind::Texlive
                } else if stdout.contains("MiKTeX") {
                    DistroKind::Miktex
                } else {
                    DistroKind::Unknown
                }
            }
            Err(_) => {
                if Command::new("tectonic")
                    .arg("--version")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .is_ok()
                {
                    DistroKind::Tectonic
                } else {
                    DistroKind::Unknown
                }
            }
        };

        let file_name_db = match kind {
            DistroKind::Texlive => {
                let root_dirs = kpsewhich::root_directories()?;
                FileNameDB::parse(&root_dirs, &mut texlive::read_database)?
            }
            DistroKind::Miktex => {
                let root_dirs = kpsewhich::root_directories()?;
                FileNameDB::parse(&root_dirs, &mut miktex::read_database)?
            }
            DistroKind::Tectonic | DistroKind::Unknown => FileNameDB::default(),
        };

        Ok(Self { kind, file_name_db })
    }
}
