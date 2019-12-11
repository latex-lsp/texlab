use super::compile::*;
use super::{Distribution, DistributionKind, LoadError, Resolver};
use futures_boxed::boxed;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Tectonic;

impl Tectonic {
    pub fn new() -> Self {
        Self
    }
}

impl Distribution for Tectonic {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Tectonic
    }

    fn supports_format(&self, format: Format) -> bool {
        match format {
            Format::Latex | Format::Pdflatex | Format::Xelatex => true,
            Format::Lualatex => false,
        }
    }

    fn output_kind(&self, _format: Format) -> OutputKind {
        OutputKind::Pdf
    }

    #[boxed]
    async fn compile<'a>(
        &'a self,
        params: CompileParams<'a>,
    ) -> Result<CompileResult, CompileError> {
        let args = [params.file_name];
        compile("tectonic", &args, params).await
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
