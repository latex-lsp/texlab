use base_db::{graph::Graph, BibDocumentData, Document, DocumentData, TexDocumentData, Workspace};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use url::Url;

use crate::{CitationError, Diagnostic, DiagnosticData, DiagnosticSource};

#[derive(Debug, Default)]
pub struct CitationErrors {
    errors: FxHashMap<Url, Vec<Diagnostic>>,
}

impl DiagnosticSource for CitationErrors {
    fn publish<'a>(
        &'a mut self,
        workspace: &'a Workspace,
        results: &mut FxHashMap<&'a Url, Vec<&'a Diagnostic>>,
    ) {
        let graphs: Vec<_> = workspace
            .iter()
            .map(|start| Graph::new(workspace, start))
            .collect();

        self.errors = Default::default();
        for document in workspace.iter() {
            let project = graphs
                .iter()
                .filter(|graph| graph.preorder().contains(&document))
                .flat_map(|graph| graph.preorder());

            if let DocumentData::Tex(data) = &document.data {
                self.process_tex(project, document, data);
            } else if let DocumentData::Bib(data) = &document.data {
                self.process_bib(project, document, data);
            }
        }

        for document in workspace.iter() {
            let Some(diagnostics) = self.errors.get(&document.uri) else { continue };

            results
                .entry(&document.uri)
                .or_default()
                .extend(diagnostics.iter());
        }
    }
}

impl CitationErrors {
    fn process_tex<'a>(
        &mut self,
        project: impl Iterator<Item = &'a Document>,
        document: &Document,
        data: &TexDocumentData,
    ) {
        let entries: FxHashSet<&str> = project
            .filter_map(|child| child.data.as_bib())
            .flat_map(|data| data.semantics.entries.iter())
            .map(|entry| entry.name.text.as_str())
            .collect();

        let mut errors = Vec::new();
        for citation in &data.semantics.citations {
            if !entries.contains(citation.name.text.as_str()) {
                errors.push(Diagnostic {
                    range: citation.name.range,
                    data: DiagnosticData::Citation(CitationError::Undefined),
                });
            }
        }

        self.errors.insert(document.uri.clone(), errors);
    }

    fn process_bib<'a>(
        &mut self,
        project: impl Iterator<Item = &'a Document>,
        document: &Document,
        data: &BibDocumentData,
    ) {
        let citations: FxHashSet<&str> = project
            .filter_map(|child| child.data.as_tex())
            .flat_map(|data| data.semantics.citations.iter())
            .map(|entry| entry.name.text.as_str())
            .collect();

        let mut errors = Vec::new();
        for entry in &data.semantics.entries {
            if !citations.contains(entry.name.text.as_str()) {
                errors.push(Diagnostic {
                    range: entry.name.range,
                    data: DiagnosticData::Citation(CitationError::Unused),
                });
            }
        }

        self.errors.insert(document.uri.clone(), errors);
    }
}
