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
use async_trait::async_trait;
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

#[async_trait]
pub trait Distribution {
    fn kind(&self) -> DistributionKind;

    async fn compile<'a>(&'a self, params: CompileParams<'a>) -> Result<Artifacts, CompileError>;

    async fn load(&self) -> Result<(), KpsewhichError>;

    async fn resolver(&self) -> Arc<Resolver>;
}

impl dyn Distribution {
    pub async fn detect() -> Arc<dyn Distribution + Send + Sync> {
        let kind = DistributionKind::detect().await;
        let distro: Arc<dyn Distribution + Send + Sync> = match kind {
            DistributionKind::Texlive => Arc::new(Texlive::new()),
            DistributionKind::Miktex => Arc::new(Miktex::new()),
            DistributionKind::Tectonic => Arc::new(Tectonic::new()),
            DistributionKind::Unknown => Arc::new(UnknownDistribution::new()),
        };
        distro
    }
}

async fn compile(params: CompileParams<'_>) -> Result<Artifacts, CompileError> {
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

#[async_trait]
impl Distribution for UnknownDistribution {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Unknown
    }

    async fn compile<'a>(&'a self, _params: CompileParams<'a>) -> Result<Artifacts, CompileError> {
        Err(CompileError::NotInstalled)
    }

    async fn load(&self) -> Result<(), KpsewhichError> {
        Ok(())
    }

    async fn resolver(&self) -> Arc<Resolver> {
        Arc::clone(&self.resolver)
    }
}

#[derive(Clone)]
pub struct DynamicDistribution(pub Arc<dyn Distribution + Send + Sync>);

impl DynamicDistribution {
    pub async fn detect() -> Self {
        Self(Distribution::detect().await)
    }
}

impl Default for DynamicDistribution {
    fn default() -> Self {
        Self(Arc::new(UnknownDistribution::new()))
    }
}

impl fmt::Debug for DynamicDistribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.kind())
    }
}
