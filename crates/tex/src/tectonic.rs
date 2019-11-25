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
        compile("tectonic", &[params.file_name], params).await
    }
}
