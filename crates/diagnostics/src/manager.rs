use base_db::{
    deps::Project, util::filter_regex_patterns, Document, DocumentData, Owner, Workspace,
};
use multimap::MultiMap;
use rowan::TextRange;
use rustc_hash::{FxHashMap, FxHashSet};
use url::Url;

use crate::types::Diagnostic;

/// Manages all diagnostics for a workspace.
#[derive(Debug, Default)]
pub struct Manager {
    grammar: MultiMap<Url, Diagnostic>,
    chktex: FxHashMap<Url, Vec<Diagnostic>>,
    build_log: FxHashMap<Url, MultiMap<Url, Diagnostic>>,
}

impl Manager {
    /// Updates the syntax-based diagnostics for the given document.
    pub fn update_syntax(&mut self, workspace: &Workspace, document: &Document) {
        if !Self::is_relevant_document(document) {
            return;
        }

        self.grammar.remove(&document.uri);
        super::grammar::tex::update(document, workspace.config(), &mut self.grammar);
        super::grammar::bib::update(document, &mut self.grammar);

        self.build_log.remove(&document.uri);
        super::build_log::update(workspace, document, &mut self.build_log);
    }

    /// Updates the ChkTeX diagnostics for the given document.
    pub fn update_chktex(&mut self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.chktex.insert(uri, diagnostics);
    }

    /// Removes stale diagnostics for documents that are no longer part of the workspace.
    pub fn cleanup(&mut self, workspace: &Workspace) {
        let uris = workspace
            .iter()
            .map(|doc| &doc.uri)
            .collect::<FxHashSet<_>>();

        self.grammar.retain(|uri, _| uris.contains(uri));
        self.chktex.retain(|uri, _| uris.contains(uri));
        self.build_log.retain(|uri, _| uris.contains(uri));
    }

    /// Returns all filtered diagnostics for the given workspace.
    pub fn get(&self, workspace: &Workspace) -> FxHashMap<Url, Vec<Diagnostic>> {
        let mut results: FxHashMap<Url, Vec<Diagnostic>> = FxHashMap::default();
        for (uri, diagnostics) in &self.grammar {
            results
                .entry(uri.clone())
                .or_default()
                .extend(diagnostics.iter().cloned());
        }

        for (uri, diagnostics) in self.build_log.values().flatten() {
            results
                .entry(uri.clone())
                .or_default()
                .extend(diagnostics.iter().cloned());
        }

        for (uri, diagnostics) in &self.chktex {
            if workspace
                .lookup(uri)
                .map_or(false, |document| document.owner == Owner::Client)
            {
                results
                    .entry(uri.clone())
                    .or_default()
                    .extend(diagnostics.iter().cloned());
            }
        }

        for document in workspace
            .iter()
            .filter(|document| Self::is_relevant_document(document))
        {
            let project = Project::from_child(workspace, document);
            super::citations::detect_undefined_citations(&project, document, &mut results);
            super::citations::detect_unused_entries(&project, document, &mut results);
        }

        super::citations::detect_duplicate_entries(workspace, &mut results);
        super::labels::detect_duplicate_labels(workspace, &mut results);
        super::labels::detect_undefined_and_unused_labels(workspace, &mut results);

        results.retain(|uri, _| {
            workspace
                .lookup(uri)
                .map_or(false, Self::is_relevant_document)
        });

        for (uri, diagnostics) in results.iter_mut() {
            diagnostics
                .retain_mut(|diagnostic| Self::filter_diagnostic(workspace, uri, diagnostic));
        }

        results
    }

    fn is_relevant_document(document: &Document) -> bool {
        match document.owner {
            Owner::Client => true,
            Owner::Server => true,
            Owner::Distro => false,
        }
    }

    fn filter_diagnostic(workspace: &Workspace, uri: &Url, diagnostic: &mut Diagnostic) -> bool {
        let config = &workspace.config().diagnostics;

        if !filter_regex_patterns(
            diagnostic.message(),
            &config.allowed_patterns,
            &config.ignored_patterns,
        ) {
            return false;
        }

        let Some(document) = workspace.lookup(uri) else {
            return false;
        };

        let Some(primary_range) = diagnostic.range(&document.line_index) else {
            return false;
        };

        if Self::is_ignored(workspace, &document.uri, &primary_range) {
            return false;
        }

        let Some(additional_locations) = diagnostic.additional_locations_mut() else {
            return true;
        };

        additional_locations.retain(|(uri, range)| !Self::is_ignored(workspace, uri, range));
        if additional_locations.is_empty() {
            return false;
        }

        true
    }

    fn is_ignored(workspace: &Workspace, uri: &Url, diag_range: &TextRange) -> bool {
        let Some(document) = workspace.lookup(uri) else {
            return false;
        };

        let DocumentData::Tex(data) = &document.data else {
            return false;
        };

        let diag_line_col = document.line_index.line_col(diag_range.start());
        let diag_offset = diag_range.start();

        let is_single_line_suppressed = data
            .semantics
            .diagnostic_suppressions
            .iter()
            .map(|r| document.line_index.line_col(r.start()))
            .any(|r| r.line == diag_line_col.line || r.line + 1 == diag_line_col.line);

        if is_single_line_suppressed {
            return true;
        }

        let is_in_suppression_range =
            data.semantics
                .warning_suppression_ranges
                .iter()
                .any(|(start, end)| {
                    let start_line = document.line_index.line_col(start.start()).line;
                    let end_offset = end.end();
                    diag_line_col.line > start_line && diag_offset <= end_offset
                });

        if is_in_suppression_range {
            return true;
        }

        false
    }
}
