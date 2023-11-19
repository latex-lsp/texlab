use std::borrow::Cow;

use base_db::{
    semantics::{bib::Entry, tex::Citation},
    util::queries::{self, Object},
    Document, Project, Workspace,
};
use rustc_hash::FxHashSet;

use crate::{
    types::{BibError, Diagnostic, DiagnosticData, TexError},
    DiagnosticBuilder, DiagnosticSource,
};

const MAX_UNUSED_ENTRIES: usize = 1000;

#[derive(Default)]
pub struct CitationErrors;

impl DiagnosticSource for CitationErrors {
    fn publish<'db>(
        &'db mut self,
        workspace: &'db Workspace,
        builder: &mut DiagnosticBuilder<'db>,
    ) {
        for document in workspace.iter() {
            let project = workspace.project(document);
            detect_undefined_citations(&project, document, builder);
            detect_unused_entries(&project, document, builder);
        }

        detect_duplicate_entries(workspace, builder);
    }
}

fn detect_undefined_citations<'db>(
    project: &Project<'db>,
    document: &'db Document,
    builder: &mut DiagnosticBuilder<'db>,
) {
    let Some(data) = document.data.as_tex() else {
        return;
    };

    let entries: FxHashSet<&str> = Entry::find_all(project)
        .map(|(_, entry)| entry.name_text())
        .collect();

    for citation in &data.semantics.citations {
        let name = citation.name_text();
        if name != "*" && !entries.contains(name) {
            let diagnostic = Diagnostic {
                range: citation.name.range,
                data: DiagnosticData::Tex(TexError::UndefinedCitation),
            };

            builder.push(&document.uri, Cow::Owned(diagnostic));
        }
    }
}

fn detect_unused_entries<'db>(
    project: &Project<'db>,
    document: &'db Document,
    builder: &mut DiagnosticBuilder<'db>,
) {
    let Some(data) = document.data.as_bib() else {
        return;
    };

    // If this is a huge bibliography, then don't bother checking for unused entries.
    if data.semantics.entries.len() > MAX_UNUSED_ENTRIES {
        return;
    }

    let citations: FxHashSet<&str> = Citation::find_all(project)
        .map(|(_, citation)| citation.name_text())
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

fn detect_duplicate_entries<'db>(workspace: &'db Workspace, builder: &mut DiagnosticBuilder<'db>) {
    for conflict in queries::Conflict::find_all::<Entry>(workspace) {
        let others = conflict
            .rest
            .iter()
            .map(|location| (location.document.uri.clone(), location.range))
            .collect();

        let diagnostic = Diagnostic {
            range: conflict.main.range,
            data: DiagnosticData::Bib(BibError::DuplicateEntry(others)),
        };

        builder.push(&conflict.main.document.uri, Cow::Owned(diagnostic));
    }
}
