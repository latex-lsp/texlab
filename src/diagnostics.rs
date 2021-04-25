mod bibtex;
mod build_log;
mod latex;

use std::sync::Arc;

use lsp_types::Diagnostic;
use multimap::MultiMap;
use rustc_hash::FxHashMap;

use crate::{Uri, Workspace};

use self::{bibtex::analyze_bibtex, build_log::analyze_build_log, latex::analyze_latex};

#[derive(Default)]
pub struct DiagnosticsManager {
    inner: FxHashMap<Arc<Uri>, MultiMap<Arc<Uri>, Diagnostic>>,
}

impl DiagnosticsManager {
    pub fn update(&mut self, workspace: &dyn Workspace, uri: Arc<Uri>) {
        let mut diagnostics_by_uri = MultiMap::new();
        analyze_build_log(workspace, &mut diagnostics_by_uri, &uri);
        analyze_bibtex(workspace, &mut diagnostics_by_uri, &uri);
        analyze_latex(workspace, &mut diagnostics_by_uri, &uri);
        self.inner.insert(uri, diagnostics_by_uri);
    }

    pub fn publish(&self, uri: Arc<Uri>) -> Vec<Diagnostic> {
        let mut all_diagnostics = Vec::new();
        for diagnostics_by_uri in self.inner.values() {
            if let Some(diagnostics) = diagnostics_by_uri.get_vec(&uri) {
                all_diagnostics.append(&mut diagnostics.clone());
            }
        }
        all_diagnostics
    }
}
