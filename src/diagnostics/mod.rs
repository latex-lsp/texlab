mod bibtex;
mod build;

use crate::diagnostics::bibtex::BibtexDiagnosticsProvider;
use crate::diagnostics::build::BuildDiagnosticsProvider;
use crate::workspace::Document;
use lsp_types::Diagnostic;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DiagnosticsManager {
    pub build: BuildDiagnosticsProvider,
    pub bibtex: BibtexDiagnosticsProvider,
}

impl DiagnosticsManager {
    pub fn get(&self, document: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        diagnostics.append(&mut self.build.get(document));
        diagnostics.append(&mut self.bibtex.get(document));
        diagnostics
    }
}
