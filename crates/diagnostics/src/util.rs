use std::borrow::Cow;

use base_db::Workspace;
use rustc_hash::FxHashMap;
use url::Url;

use crate::{Diagnostic, DiagnosticBuilder, DiagnosticSource};

#[derive(Default)]
pub struct SimpleDiagnosticSource {
    pub errors: FxHashMap<Url, Vec<Diagnostic>>,
}

impl DiagnosticSource for SimpleDiagnosticSource {
    fn publish<'db>(
        &'db mut self,
        workspace: &'db Workspace,
        builder: &mut DiagnosticBuilder<'db>,
    ) {
        self.errors.retain(|uri, _| workspace.lookup(uri).is_some());

        for document in workspace.iter() {
            if let Some(diagnostics) = self.errors.get(&document.uri) {
                builder.push_many(&document.uri, diagnostics.iter().map(Cow::Borrowed));
            }
        }
    }
}
