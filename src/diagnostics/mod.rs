mod bibtex;
mod build_log;
mod chktex;
mod debouncer;
mod latex;

use std::sync::Arc;

use lsp_types::{Diagnostic, Url};
use multimap::MultiMap;
use rustc_hash::FxHashMap;

use crate::{Options, Workspace};

pub use self::debouncer::{DiagnosticsDebouncer, DiagnosticsMessage};

use self::{
    bibtex::analyze_bibtex_static, build_log::analyze_build_log_static,
    chktex::analyze_latex_chktex, latex::analyze_latex_static,
};

#[derive(Default)]
pub struct DiagnosticsManager {
    static_diagnostics: FxHashMap<Arc<Url>, MultiMap<Arc<Url>, Diagnostic>>,
    chktex_diagnostics: MultiMap<Arc<Url>, Diagnostic>,
}

impl DiagnosticsManager {
    pub fn update_static(&mut self, workspace: &Workspace, uri: Arc<Url>) {
        let mut diagnostics_by_uri = MultiMap::new();
        analyze_build_log_static(workspace, &mut diagnostics_by_uri, &uri);
        analyze_bibtex_static(workspace, &mut diagnostics_by_uri, &uri);
        analyze_latex_static(workspace, &mut diagnostics_by_uri, &uri);
        self.static_diagnostics.insert(uri, diagnostics_by_uri);
    }

    pub fn update_chktex(&mut self, workspace: &Workspace, uri: Arc<Url>, options: &Options) {
        analyze_latex_chktex(workspace, &mut self.chktex_diagnostics, &uri, options);
    }

    pub fn publish(&self, uri: Arc<Url>) -> Vec<Diagnostic> {
        let mut all_diagnostics = Vec::new();
        for diagnostics_by_uri in self.static_diagnostics.values() {
            if let Some(diagnostics) = diagnostics_by_uri.get_vec(&uri) {
                all_diagnostics.append(&mut diagnostics.clone());
            }
        }

        if let Some(diagnostics) = self.chktex_diagnostics.get_vec(&uri) {
            all_diagnostics.append(&mut diagnostics.clone());
        }

        all_diagnostics
    }
}
