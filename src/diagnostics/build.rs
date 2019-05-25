use crate::build::log_parser::parse_build_log;
use lsp_types::{Diagnostic, Uri};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct BuildDiagnosticsProvider {
    diagnostics_by_uri: HashMap<Uri, Vec<Diagnostic>>,
}

impl BuildDiagnosticsProvider {
    pub fn get(&self, uri: &Uri) -> &[Diagnostic] {
        match self.diagnostics_by_uri.get(uri) {
            Some(diagnostics) => diagnostics,
            None => &[],
        }
    }

    pub fn update(&mut self, uri: &Uri, log: &str) {
        self.diagnostics_by_uri.clear();
        for error in parse_build_log(uri, log) {
            let diagnostics = self
                .diagnostics_by_uri
                .entry(error.uri.clone())
                .or_insert_with(|| Vec::new());
            diagnostics.push(error.into());
        }
    }
}
