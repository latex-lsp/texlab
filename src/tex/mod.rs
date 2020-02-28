mod compile;
mod kpsewhich;
mod miktex;
mod tectonic;
mod texlive;

pub use self::{
    compile::{Artifacts, CompileError, CompileParams, Format},
    kpsewhich::{KpsewhichError, Resolver},
};

use self::{compile::Compiler, miktex::Miktex, tectonic::Tectonic, texlive::Texlive};
use futures_boxed::boxed;
use std::{fmt, process::Stdio, sync::Arc};
use tokio::process::Command;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DistributionKind {
    Texlive,
    Miktex,
    Tectonic,
    Unknown,
}

impl fmt::Display for DistributionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Texlive => "TeXLive",
            Self::Miktex => "MikTeX",
            Self::Tectonic => "Tectonic",
            Self::Unknown => "Unknown",
        };
        write!(f, "{}", name)
    }
}

impl DistributionKind {
    pub async fn detect() -> Self {
        if Command::new("tectonic")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .is_ok()
        {
            return Self::Tectonic;
        }

        match Command::new("latex").arg("--version").output().await {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("TeX Live") {
                    Self::Texlive
                } else if stdout.contains("MiKTeX") {
                    Self::Miktex
                } else {
                    Self::Unknown
                }
            }
            Err(_) => Self::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Language {
    Latex,
    Bibtex,
}

impl Language {
    pub fn by_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "tex" | "sty" | "cls" | "def" | "lco" | "aux" => Some(Language::Latex),
            "bib" | "bibtex" => Some(Language::Bibtex),
            _ => None,
        }
    }

    pub fn by_language_id(language_id: &str) -> Option<Self> {
        match language_id {
            "latex" | "tex" => Some(Language::Latex),
            "bibtex" | "bib" => Some(Language::Bibtex),
            _ => None,
        }
    }
}

pub trait Distribution {
    fn kind(&self) -> DistributionKind;

    #[boxed]
    async fn compile<'a>(&'a self, params: CompileParams<'a>) -> Result<Artifacts, CompileError> {
        let executable = params.format.executable();
        let args = &["--interaction=batchmode", "-shell-escape", params.file_name];
        let compiler = Compiler {
            executable,
            args,
            file_name: params.file_name,
            timeout: params.timeout,
        };
        compiler.compile(params.code).await
    }

    #[boxed]
    async fn load(&self) -> Result<(), KpsewhichError>;

    #[boxed]
    async fn resolver(&self) -> Arc<Resolver>;
}

impl dyn Distribution {
    pub async fn detect() -> Box<dyn Distribution + Send + Sync> {
        let kind = DistributionKind::detect().await;
        let distro: Box<dyn Distribution + Send + Sync> = match kind {
            DistributionKind::Texlive => Box::new(Texlive::new()),
            DistributionKind::Miktex => Box::new(Miktex::new()),
            DistributionKind::Tectonic => Box::new(Tectonic::new()),
            DistributionKind::Unknown => Box::new(UnknownDistribution::new()),
        };
        distro
    }
}

#[derive(Debug, Default)]
pub struct UnknownDistribution {
    resolver: Arc<Resolver>,
}

impl UnknownDistribution {
    pub fn new() -> Self {
        Self {
            resolver: Arc::default(),
        }
    }
}

impl Distribution for UnknownDistribution {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Unknown
    }

    #[boxed]
    async fn compile<'a>(&'a self, _params: CompileParams) -> Result<Artifacts, CompileError> {
        Err(CompileError::NotInstalled)
    }

    #[boxed]
    async fn load(&self) -> Result<(), KpsewhichError> {
        Ok(())
    }

    #[boxed]
    async fn resolver(&self) -> Arc<Resolver> {
        Arc::clone(&self.resolver)
    }
}
