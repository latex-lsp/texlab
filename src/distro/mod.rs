mod kpsewhich;
mod miktex;
mod texlive;

use std::process::{Command, Stdio};

use derive_more::Display;
use log::warn;

pub use self::kpsewhich::{KpsewhichError, Resolver};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum DistroKind {
    #[display(fmt = "TeXLive")]
    Texlive,
    #[display(fmt = "MikTeX")]
    Miktex,
    #[display(fmt = "Tectonic")]
    Tectonic,
    #[display(fmt = "Unknown")]
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Distro {
    pub kind: DistroKind,
    pub resolver: Resolver,
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

        let resolver = match kind {
            DistroKind::Texlive => Self::load_resolver(texlive::load_resolver),
            DistroKind::Miktex => Self::load_resolver(miktex::load_resolver),
            DistroKind::Tectonic | DistroKind::Unknown => Resolver::default(),
        };
        Self { kind, resolver }
    }

    fn load_resolver(loader: impl FnOnce() -> Result<Resolver, KpsewhichError>) -> Resolver {
        match loader() {
            Ok(resolver) => return resolver,
            Err(KpsewhichError::NotInstalled) | Err(KpsewhichError::InvalidOutput) => {
                warn!(
                    "An error occurred while executing `kpsewhich`.\
                     Please make sure that your distribution is in your PATH \
                     environment variable and provides the `kpsewhich` tool."
                );
            }
            Err(KpsewhichError::CorruptDatabase) | Err(KpsewhichError::NoDatabase) => {
                warn!(
                    "The file database of your TeX distribution seems \
                     to be corrupt. Please rebuild it and try again."
                );
            }
            Err(KpsewhichError::IO(why)) => {
                warn!("An I/O error occurred while executing 'kpsewhich': {}", why);
            }
            Err(KpsewhichError::Decode(_)) => {
                warn!("An error occurred while decoding the output of `kpsewhich`.");
            }
        };
        Resolver::default()
    }
}
