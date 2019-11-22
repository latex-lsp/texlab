use super::compile::*;
use super::{Distribution, DistributionKind};

#[derive(Debug, Default)]
pub struct Texlive;

impl Distribution for Texlive {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Texlive
    }

    fn supports_format(&self, format: Format) -> bool {
        match format {
            Format::Latex | Format::Pdflatex => true,
            Format::Xelatex | Format::Lualatex => true,
        }
    }
}
