mod bibtex;
mod build;

pub use self::{
    bibtex::{BibtexDiagnosticsProvider, BibtexError, BibtexErrorCode},
    build::BuildDiagnosticsProvider,
};

use crate::{protocol::Diagnostic, workspace::Document};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DiagnosticsManager {
    pub bibtex: BibtexDiagnosticsProvider,
    pub build: BuildDiagnosticsProvider,
}

impl DiagnosticsManager {
    pub fn get(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        diagnostics.append(&mut self.bibtex.get(doc));
        diagnostics.append(&mut self.build.get(doc));
        diagnostics
    }
}
