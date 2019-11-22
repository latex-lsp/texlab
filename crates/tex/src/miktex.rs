use super::compile::*;
use super::{Distribution, DistributionKind};

#[derive(Debug, Default)]
pub struct Miktex;

impl Distribution for Miktex {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Miktex
    }

    fn supports_format(&self, format: Format) -> bool {
        match format {
            Format::Latex | Format::Pdflatex => true,
            Format::Xelatex | Format::Lualatex => true,
        }
    }
}
