use base_db::Workspace;
use rustc_hash::FxHashMap;
use url::Url;

use crate::{Diagnostic, DiagnosticSource};

#[derive(Default)]
pub struct SimpleDiagnosticSource {
    pub errors: FxHashMap<Url, Vec<Diagnostic>>,
}

impl DiagnosticSource for SimpleDiagnosticSource {
    fn publish<'a>(
        &'a mut self,
        workspace: &'a Workspace,
        results: &mut FxHashMap<&'a Url, Vec<&'a Diagnostic>>,
    ) {
        self.errors.retain(|uri, _| workspace.lookup(uri).is_some());

        for document in workspace.iter() {
            let Some(diagnostics) = self.errors.get(&document.uri) else { continue };

            results
                .entry(&document.uri)
                .or_default()
                .extend(diagnostics.iter());
        }
    }
}
