use std::borrow::Cow;

use base_db::{graph::Graph, BibDocumentData, Document, DocumentData, TexDocumentData, Workspace};
use itertools::Itertools;
use rustc_hash::FxHashSet;

use crate::{
    types::{BibError, Diagnostic, DiagnosticData, TexError},
    DiagnosticBuilder, DiagnosticSource,
};

#[derive(Default)]
pub struct CitationErrors;

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

        for document in workspace.iter() {
            let project = graphs
                .iter()
                .filter(|graph| graph.preorder().contains(&document))
                .flat_map(|graph| graph.preorder());

            if let DocumentData::Tex(data) = &document.data {
                self.process_tex(project, document, data, builder);
            } else if let DocumentData::Bib(data) = &document.data {
                self.process_bib(project, document, data, builder);
            }
        }
    }
}

impl CitationErrors {
    fn process_tex<'db>(
        &mut self,
        project: impl Iterator<Item = &'db Document>,
        document: &'db Document,
        data: &TexDocumentData,
        builder: &mut DiagnosticBuilder<'db>,
    ) {
        let entries: FxHashSet<&str> = project
            .filter_map(|child| child.data.as_bib())
            .flat_map(|data| data.semantics.entries.iter())
            .map(|entry| entry.name.text.as_str())
            .collect();

        for citation in &data.semantics.citations {
            if !entries.contains(citation.name.text.as_str()) {
                let diagnostic = Diagnostic {
                    range: citation.name.range,
                    data: DiagnosticData::Tex(TexError::UndefinedCitation),
                };

                builder.push(&document.uri, Cow::Owned(diagnostic));
            }
        }
    }

    fn process_bib<'db>(
        &mut self,
        project: impl Iterator<Item = &'db Document>,
        document: &'db Document,
        data: &BibDocumentData,
        builder: &mut DiagnosticBuilder<'db>,
    ) {
        let citations: FxHashSet<&str> = project
            .filter_map(|child| child.data.as_tex())
            .flat_map(|data| data.semantics.citations.iter())
            .map(|entry| entry.name.text.as_str())
            .collect();

        for entry in &data.semantics.entries {
            if !citations.contains(entry.name.text.as_str()) {
                let diagnostic = Diagnostic {
                    range: entry.name.range,
                    data: DiagnosticData::Bib(BibError::UnusedEntry),
                };

                builder.push(&document.uri, Cow::Owned(diagnostic));
            }
        }
    }
}
