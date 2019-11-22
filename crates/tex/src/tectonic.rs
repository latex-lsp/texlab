use super::compile::*;
use super::{Distribution, DistributionKind};
use futures_boxed::boxed;

#[derive(Debug, Default)]
pub struct Tectonic;

impl Distribution for Tectonic {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Tectonic
    }

    fn supports_format(&self, format: Format) -> bool {
        match format {
            Format::Latex | Format::Lualatex => false,
            Format::Pdflatex | Format::Xelatex => true,
        }
    }

    #[boxed]
    async fn compile<'a>(
        &'a self,
        params: CompileParams<'a>,
    ) -> Result<CompileResult, CompileError> {
        compile("tectonic", &[params.file_name], params).await
    }
}
