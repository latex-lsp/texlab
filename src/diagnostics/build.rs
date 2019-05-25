use crate::build::log_parser::parse_build_log;
use crate::feature::FeatureRequest;
use crate::workspace::Document;
use lsp_types::{Diagnostic, Uri};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct BuildDiagnosticsProvider {
    diagnostics_by_uri: HashMap<Uri, Vec<Diagnostic>>,
}

impl BuildDiagnosticsProvider {
    pub fn get(&self, document: &Document) -> Vec<Diagnostic> {
        match self.diagnostics_by_uri.get(&document.uri) {
            Some(diagnostics) => diagnostics.to_owned(),
            None => Vec::new(),
        }
    }

    pub fn update(&mut self, uri: &Uri, log: &str) {
        self.diagnostics_by_uri.clear();
        for error in parse_build_log(uri, log) {
            let diagnostics = self
                .diagnostics_by_uri
                .entry(error.uri.clone())
                .or_insert_with(Vec::new);
            diagnostics.push(error.into());
        }
    }
}
