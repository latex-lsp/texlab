use crate::workspace::Uri;
use lsp_types::ProgressToken;
use std::mem;
use std::sync::Mutex;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LintReason {
    Change,
    Save,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Action {
    CheckInstalledDistribution,
    DetectRoot(Uri),
    PublishDiagnostics,
    RunLinter(Uri, LintReason),
    Build(Uri),
    CancelBuild(ProgressToken),
}

#[derive(Debug, Default)]
pub struct ActionMananger {
    actions: Mutex<Vec<Action>>,
}

impl ActionMananger {
    pub fn push(&self, action: Action) {
        let mut actions = self.actions.lock().unwrap();
        actions.push(action);
    }

    pub fn take(&self) -> Vec<Action> {
        let mut actions = self.actions.lock().unwrap();
        mem::replace(&mut *actions, Vec::new())
    }
}
