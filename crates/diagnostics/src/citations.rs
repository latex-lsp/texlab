use base_db::{
    deps::Project,
    semantics::{bib::Entry, tex::Citation},
    util::queries::{self, Object},
    Document, Workspace,
};
use multimap::MultiMap;
use rustc_hash::FxHashSet;
use url::Url;

use crate::types::{BibError, Diagnostic, TexError};

const MAX_UNUSED_ENTRIES: usize = 1000;

pub fn detect_undefined_citations<'a>(
    project: &Project<'a>,
    document: &'a Document,
    results: &mut MultiMap<Url, Diagnostic>,
) -> Option<()> {
    let data = document.data.as_tex()?;

    let entries: FxHashSet<&str> = Entry::find_all(project)
        .map(|(_, entry)| entry.name_text())
        .collect();

    for citation in &data.semantics.citations {
        let name = citation.name_text();
        if name != "*" && !entries.contains(name) {
            let diagnostic = Diagnostic::Tex(citation.name.range, TexError::UndefinedCitation);
            results.insert(document.uri.clone(), diagnostic);
        }
    }

    Some(())
}

pub fn detect_unused_entries<'a>(
    project: &Project<'a>,
    document: &'a Document,
    results: &mut MultiMap<Url, Diagnostic>,
) -> Option<()> {
    let data = document.data.as_bib()?;

    // If this is a huge bibliography, then don't bother checking for unused entries.
    if data.semantics.entries.len() > MAX_UNUSED_ENTRIES {
        return None;
    }

    let citations: FxHashSet<&str> = Citation::find_all(project)
        .map(|(_, citation)| citation.name_text())
        .collect();

    for entry in &data.semantics.entries {
        if !citations.contains(entry.name.text.as_str()) {
            let diagnostic = Diagnostic::Bib(entry.name.range, BibError::UnusedEntry);
            results.insert(document.uri.clone(), diagnostic);
        }
    }

    Some(())
}

pub fn detect_duplicate_entries<'a>(
    workspace: &'a Workspace,
    results: &mut MultiMap<Url, Diagnostic>,
) {
    for conflict in queries::Conflict::find_all::<Entry>(workspace) {
        let others = conflict
            .rest
            .iter()
            .map(|location| (location.document.uri.clone(), location.range))
            .collect();

        let diagnostic = Diagnostic::Bib(conflict.main.range, BibError::DuplicateEntry(others));
        results.insert(conflict.main.document.uri.clone(), diagnostic);
    }
}
