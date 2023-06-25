use base_db::{graph::Graph, BibDocumentData, Document, DocumentData, TexDocumentData, Workspace};
use itertools::Itertools;
use rustc_hash::FxHashSet;

use crate::{
    types::{CitationError, Diagnostic, DiagnosticData},
    util::SimpleDiagnosticSource,
    DiagnosticBuilder, DiagnosticSource,
};

#[derive(Default)]
pub struct CitationErrors(SimpleDiagnosticSource);

impl DiagnosticSource for CitationErrors {
    fn publish<'db>(
        &'db mut self,
        workspace: &'db Workspace,
        builder: &mut DiagnosticBuilder<'db>,
    ) {
        let graphs: Vec<_> = workspace
            .iter()
            .map(|start| Graph::new(workspace, start))
            .collect();

        self.0 = Default::default();
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

        self.0.publish(workspace, builder);
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

        self.0.errors.insert(document.uri.clone(), errors);
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

        self.0.errors.insert(document.uri.clone(), errors);
    }
}
