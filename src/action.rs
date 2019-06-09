use lsp_types::Uri;
use std::mem;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Action {
    RegisterCapabilities,
    LoadResolver,
    ResolveIncludes,
    PublishDiagnostics,
    RunLinter(Uri),
    ParseLog { tex_uri: Uri, log_path: PathBuf },
    Build(Uri),
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
