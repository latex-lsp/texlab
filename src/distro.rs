mod kpsewhich;
mod miktex;
mod texlive;

use std::process::{Command, Stdio};

use anyhow::Result;
use derive_more::Display;
use log::warn;

pub use kpsewhich::Resolver;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum DistributionKind {
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
pub struct Distribution {
    pub kind: DistributionKind,
    pub resolver: Resolver,
}

impl Distribution {
    pub fn detect() -> Self {
        let kind = match Command::new("latex").arg("--version").output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("TeX Live") {
                    DistributionKind::Texlive
                } else if stdout.contains("MiKTeX") {
                    DistributionKind::Miktex
                } else {
                    DistributionKind::Unknown
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
                    DistributionKind::Tectonic
                } else {
                    DistributionKind::Unknown
                }
            }
        };

        let resolver = match kind {
            DistributionKind::Texlive => Self::load_resolver(texlive::load_resolver),
            DistributionKind::Miktex => Self::load_resolver(miktex::load_resolver),
            DistributionKind::Tectonic | DistributionKind::Unknown => Resolver::default(),
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
