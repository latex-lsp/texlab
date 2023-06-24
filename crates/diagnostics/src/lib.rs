mod build_log;
mod citations;
mod grammar;
mod labels;
pub(crate) mod util;

use base_db::{Document, Workspace};
use build_log::BuildErrors;
use citations::CitationErrors;
use grammar::{BibSyntaxErrors, TexSyntaxErrors};
use labels::LabelErrors;
use rowan::TextRange;
use rustc_hash::FxHashMap;
use syntax::BuildError;
use url::Url;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: TextRange,
    pub data: DiagnosticData,
}

#[derive(Debug, Clone)]
pub enum DiagnosticData {
    Syntax(SyntaxError),
    Build(BuildError),
    Label(LabelError),
    Citation(CitationError),
}

#[derive(Debug, Clone)]
pub enum SyntaxError {
    UnexpectedRCurly,
    RCurlyInserted,
    MismatchedEnvironment,
    ExpectingLCurly,
    ExpectingKey,
    ExpectingRCurly,
    ExpectingEq,
    ExpectingFieldValue,
}

#[derive(Debug, Clone)]
pub enum LabelError {
    Unused,
    Undefined,
}

#[derive(Debug, Clone)]
pub enum CitationError {
    Unused,
    Undefined,
}

pub trait DiagnosticSource {
    fn update(&mut self, _workspace: &Workspace, _document: &Document) {}

    fn publish<'a>(
        &'a mut self,
        workspace: &'a Workspace,
        results: &mut FxHashMap<&'a Url, Vec<&'a Diagnostic>>,
    );
}

pub struct DiagnosticManager {
    sources: Vec<Box<dyn DiagnosticSource>>,
}

impl Default for DiagnosticManager {
    fn default() -> Self {
        let mut sources: Vec<Box<dyn DiagnosticSource>> = Vec::new();
        sources.push(Box::new(TexSyntaxErrors::default()));
        sources.push(Box::new(BibSyntaxErrors::default()));
        sources.push(Box::new(BuildErrors::default()));
        sources.push(Box::new(LabelErrors::default()));
        sources.push(Box::new(CitationErrors::default()));
        Self { sources }
    }
}

impl DiagnosticSource for DiagnosticManager {
    fn update(&mut self, workspace: &Workspace, document: &Document) {
        for source in &mut self.sources {
            source.update(workspace, document);
        }
    }

    fn publish<'a>(
        &'a mut self,
        workspace: &'a Workspace,
        results: &mut FxHashMap<&'a Url, Vec<&'a Diagnostic>>,
    ) {
        for source in &mut self.sources {
            source.publish(workspace, results);
        }
    }
}
