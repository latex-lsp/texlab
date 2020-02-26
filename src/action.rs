use std::mem;
use std::sync::Mutex;
use texlab_protocol::{ProgressToken, Uri};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LintReason {
    Change,
    Save,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    RegisterCapabilities,
    LoadDistribution,
    LoadConfiguration,
    UpdateConfiguration(serde_json::Value),
    DetectRoot(Uri),
    PublishDiagnostics,
    RunLinter(Uri, LintReason),
    Build(Uri),
    CancelBuild(ProgressToken),
}

#[derive(Debug, Default)]
pub struct ActionManager {
    actions: Mutex<Vec<Action>>,
}

impl ActionManager {
    pub fn push(&self, action: Action) {
        let mut actions = self.actions.lock().unwrap();
        actions.push(action);
    }

    pub fn take(&self) -> Vec<Action> {
        let mut actions = self.actions.lock().unwrap();
        mem::replace(&mut *actions, Vec::new())
    }
}
