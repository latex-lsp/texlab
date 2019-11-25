mod compile;
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
use tokio_net::process::Command;

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
}

impl dyn Distribution {
    pub async fn detect() -> Box<Self> {
        let kind = DistributionKind::detect().await;
        let distro: Box<Self> = match kind {
            DistributionKind::Texlive => Box::new(Texlive),
            DistributionKind::Miktex => Box::new(Miktex),
            DistributionKind::Tectonic => Box::new(Tectonic),
            DistributionKind::Unknown => Box::new(Unknown),
        };
        distro
    }
}

#[derive(Debug, Default)]
pub struct Unknown;

impl Distribution for Unknown {
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
}
