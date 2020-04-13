mod bibtex;
mod build;
mod latex;

pub use self::{
    bibtex::{BibtexDiagnosticsProvider, BibtexError, BibtexErrorCode},
    build::BuildDiagnosticsProvider,
    latex::LatexDiagnosticsProvider,
};

use texlab_feature::Document;
use texlab_protocol::Diagnostic;

#[derive(Debug, Default)]
pub struct DiagnosticsManager {
    pub bibtex: BibtexDiagnosticsProvider,
    pub latex: LatexDiagnosticsProvider,
    pub build: BuildDiagnosticsProvider,
}

impl DiagnosticsManager {
    pub async fn get(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        diagnostics.append(&mut self.bibtex.get(doc));
        diagnostics.append(&mut self.latex.get(doc));
        diagnostics.append(&mut self.build.get(doc).await);
        diagnostics
    }
}
