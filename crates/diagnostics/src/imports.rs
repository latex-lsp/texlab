use base_db::{semantics::tex::Link, util::queries, Workspace};
use rustc_hash::FxHashMap;
use url::Url;

use crate::{types::Diagnostic, ImportError};

pub fn detect_duplicate_imports(
    workspace: &Workspace,
    results: &mut FxHashMap<Url, Vec<Diagnostic>>,
) {
    for conflict in queries::Conflict::find_all::<Link>(workspace) {
        let others = conflict
            .rest
            .iter()
            .map(|location| (location.document.uri.clone(), location.range))
            .collect();

        let diagnostic =
            Diagnostic::Import(conflict.main.range, ImportError::DuplicateImport(others));
        results
            .entry(conflict.main.document.uri.clone())
            .or_default()
            .push(diagnostic);
    }
}
