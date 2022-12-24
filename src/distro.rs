mod kpsewhich;
mod miktex;
mod texlive;

use std::process::{Command, Stdio};

use anyhow::Result;
use log::warn;

pub use kpsewhich::Resolver;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DistroKind {
    Texlive,
    Miktex,
    Tectonic,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Distro {
    pub kind: DistroKind,
    pub resolver: Resolver,
}

impl Distro {
    #[must_use]
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

        let resolver = match kind {
            DistroKind::Texlive => Self::load_resolver(texlive::load_resolver),
            DistroKind::Miktex => Self::load_resolver(miktex::load_resolver),
            DistroKind::Tectonic | DistroKind::Unknown => Resolver::default(),
        };
        Self { kind, resolver }
    }

    fn load_resolver(loader: impl FnOnce() -> Result<Resolver>) -> Resolver {
        match loader() {
            Ok(resolver) => return resolver,
            Err(why) => warn!("Failed to load resolver: {}", why),
        };
        Resolver::default()
    }
}
