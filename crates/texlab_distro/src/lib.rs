mod compile;
mod kpsewhich;
mod language;
mod miktex;
mod tectonic;
mod texlive;

pub use self::compile::*;
pub use self::language::Language;

use self::miktex::Miktex;
use self::tectonic::Tectonic;
use self::texlive::Texlive;
use futures_boxed::boxed;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DistributionKind {
    Texlive,
    Miktex,
    Tectonic,
    Unknown,
}

impl DistributionKind {
    pub async fn detect() -> Self {
        if Command::new("tectonic")
            .arg("--version")
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Resolver {
    pub files_by_name: HashMap<String, PathBuf>,
}

impl Resolver {
    pub fn new(files_by_name: HashMap<String, PathBuf>) -> Self {
        Self { files_by_name }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LoadError {
    KpsewhichNotFound,
    CorruptFileDatabase,
}

pub trait Distribution: Send + Sync {
    fn kind(&self) -> DistributionKind;

    fn supports_format(&self, format: Format) -> bool;

    fn output_kind(&self, format: Format) -> OutputKind {
        match format {
            Format::Latex => OutputKind::Dvi,
            Format::Pdflatex | Format::Xelatex | Format::Lualatex => OutputKind::Pdf,
        }
    }

    #[boxed]
    async fn compile<'a>(
        &'a self,
        params: CompileParams<'a>,
    ) -> Result<CompileResult, CompileError> {
        let executable = params.format.executable();
        let args = &["--interaction=batchmode", "-shell-escape", params.file_name];
        compile(executable, args, params).await
    }

    #[boxed]
    async fn load(&self) -> Result<(), LoadError>;

    #[boxed]
    async fn resolver(&self) -> Arc<Resolver>;
}

impl dyn Distribution {
    pub async fn detect() -> Box<Self> {
        let kind = DistributionKind::detect().await;
        let distro: Box<Self> = match kind {
            DistributionKind::Texlive => Box::new(Texlive::new()),
            DistributionKind::Miktex => Box::new(Miktex::new()),
            DistributionKind::Tectonic => Box::new(Tectonic::new()),
            DistributionKind::Unknown => Box::new(UnknownDistribution::new()),
        };
        distro
    }
}

#[derive(Debug, Default)]
pub struct UnknownDistribution;

impl UnknownDistribution {
    pub fn new() -> Self {
        Self
    }
}

impl Distribution for UnknownDistribution {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Unknown
    }

    fn supports_format(&self, _format: Format) -> bool {
        false
    }

    #[boxed]
    async fn compile<'a>(
        &'a self,
        _params: CompileParams<'a>,
    ) -> Result<CompileResult, CompileError> {
        Err(CompileError::NotInstalled)
    }

    #[boxed]
    async fn load(&self) -> Result<(), LoadError> {
        Ok(())
    }

    #[boxed]
    async fn resolver(&self) -> Arc<Resolver> {
        Arc::new(Resolver::default())
    }
}
