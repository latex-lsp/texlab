use crate::diagnostics::build::BuildDiagnosticsProvider;
use lsp_types::{Diagnostic, Uri};

mod build;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DiagnosticsManager {
    pub build: BuildDiagnosticsProvider,
}

impl DiagnosticsManager {
    pub fn get(&self, uri: &Uri) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        diagnostics.append(&mut self.build.get(uri).to_owned());
        diagnostics
    }
}
