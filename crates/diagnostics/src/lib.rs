mod build_log;
mod citations;
mod grammar;
mod labels;
pub mod types;
pub(crate) mod util;

use std::borrow::Cow;

use base_db::{Document, Workspace};
use build_log::BuildErrors;
use citations::CitationErrors;
use grammar::{BibSyntaxErrors, TexSyntaxErrors};
use labels::LabelErrors;
use rustc_hash::FxHashMap;
use types::Diagnostic;
use url::Url;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DiagnosticBuilder<'db> {
    inner: FxHashMap<&'db Url, Vec<Cow<'db, Diagnostic>>>,
}

impl<'db> DiagnosticBuilder<'db> {
    pub fn push(&mut self, uri: &'db Url, diagnostic: Cow<'db, Diagnostic>) {
        self.inner.entry(uri).or_default().push(diagnostic);
    }

    pub fn push_many(
        &mut self,
        uri: &'db Url,
        diagnostics: impl Iterator<Item = Cow<'db, Diagnostic>>,
    ) {
        self.inner.entry(uri).or_default().extend(diagnostics);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'db Url, impl Iterator<Item = &Diagnostic>)> {
        self.inner
            .iter()
            .map(|(uri, diagnostics)| (*uri, diagnostics.iter().map(|diag| diag.as_ref())))
    }
}

pub trait DiagnosticSource {
    #[allow(unused_variables)]
    fn update(&mut self, workspace: &Workspace, document: &Document) {}

    fn publish<'db>(&'db mut self, workspace: &'db Workspace, builder: &mut DiagnosticBuilder<'db>);
}

pub struct DiagnosticManager {
    sources: Vec<Box<dyn DiagnosticSource>>,
}

impl Default for DiagnosticManager {
    fn default() -> Self {
        let sources: Vec<Box<dyn DiagnosticSource>> = vec![
            Box::<TexSyntaxErrors>::default(),
            Box::<BibSyntaxErrors>::default(),
            Box::<BuildErrors>::default(),
            Box::<LabelErrors>::default(),
            Box::<CitationErrors>::default(),
        ];

        Self { sources }
    }
}

impl DiagnosticSource for DiagnosticManager {
    fn update(&mut self, workspace: &Workspace, document: &Document) {
        for source in &mut self.sources {
            source.update(workspace, document);
        }
    }

    fn publish<'db>(
        &'db mut self,
        workspace: &'db Workspace,
        builder: &mut DiagnosticBuilder<'db>,
    ) {
        for source in &mut self.sources {
            source.publish(workspace, builder);
        }
    }
}

#[cfg(test)]
mod tests;
