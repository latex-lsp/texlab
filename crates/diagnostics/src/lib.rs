pub mod build_log;
pub mod grammar;
pub mod labels;

use base_db::{Document, Workspace};
use rowan::TextRange;
use rustc_hash::FxHashMap;
use syntax::BuildError;
use url::Url;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: TextRange,
    pub code: ErrorCode,
}

#[derive(Debug, Clone)]
pub enum ErrorCode {
    UnexpectedRCurly,
    RCurlyInserted,
    MismatchedEnvironment,
    ExpectingLCurly,
    ExpectingKey,
    ExpectingRCurly,
    ExpectingEq,
    ExpectingFieldValue,
    Build(BuildError),
    Label(LabelErrorCode),
}

#[derive(Debug, Clone)]
pub enum LabelErrorCode {
    Unused,
    Undefined,
}

pub trait DiagnosticSource {
    fn on_change(&mut self, workspace: &Workspace, document: &Document);

    fn cleanup(&mut self, workspace: &Workspace);

    fn publish<'this, 'db>(
        &'this mut self,
        workspace: &'db Workspace,
        results: &mut FxHashMap<&'db Url, Vec<&'this Diagnostic>>,
    );
}

#[derive(Default)]
pub struct DiagnosticManager {
    sources: Vec<Box<dyn DiagnosticSource>>,
}

impl DiagnosticManager {
    pub fn with(mut self, source: Box<dyn DiagnosticSource>) -> Self {
        self.sources.push(source);
        self
    }
}

impl DiagnosticSource for DiagnosticManager {
    fn on_change(&mut self, workspace: &Workspace, document: &Document) {
        for source in &mut self.sources {
            source.on_change(workspace, document);
        }
    }

    fn cleanup(&mut self, workspace: &Workspace) {
        for source in &mut self.sources {
            source.cleanup(workspace);
        }
    }

    fn publish<'this, 'db>(
        &'this mut self,
        workspace: &'db Workspace,
        results: &mut FxHashMap<&'db Url, Vec<&'this Diagnostic>>,
    ) {
        for source in &mut self.sources {
            source.publish(workspace, results);
        }
    }
}
