mod file_name_db;
mod kpsewhich;
mod miktex;
mod texlive;

use std::process::{Command, Stdio};

use log::warn;

pub use file_name_db::FileNameDB;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DistroKind {
    Texlive,
    Miktex,
    Tectonic,
    Unknown,
}

#[derive(Debug)]
pub struct Distro {
    pub kind: DistroKind,
    pub file_name_db: FileNameDB,
}

impl Distro {
    pub fn detect() -> Self {
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
            DistroKind::Texlive => kpsewhich::root_directories()
                .and_then(|root_dirs| FileNameDB::parse(&root_dirs, &mut texlive::read_database)),
            DistroKind::Miktex => kpsewhich::root_directories()
                .and_then(|root_dirs| FileNameDB::parse(&root_dirs, &mut miktex::read_database)),
            DistroKind::Tectonic | DistroKind::Unknown => Ok(FileNameDB::default()),
        };

        let file_name_db = file_name_db.unwrap_or_else(|why| {
            warn!("Failed to load distro files: {}", why);
            FileNameDB::default()
        });

        Self { kind, file_name_db }
    }
}
