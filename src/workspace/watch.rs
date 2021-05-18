use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use log::warn;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rustc_hash::FxHashSet;

use crate::{
    Document, DocumentLanguage, OpenHandler, Uri, Workspace, WorkspaceSource, WorkspaceSubset,
};

pub struct DocumentWatcher<W> {
    workspace: Arc<W>,
    watcher: Mutex<RecommendedWatcher>,
    watched_paths: Mutex<FxHashSet<PathBuf>>,
}

impl<W> DocumentWatcher<W>
where
    W: Workspace + Send + Sync + 'static,
{
    pub fn new(workspace: Arc<W>) -> Result<Self> {
        let watcher = Self::create_watcher(Arc::clone(&workspace))?;
        Ok(Self {
            workspace,
            watcher: Mutex::new(watcher),
            watched_paths: Mutex::default(),
        })
    }

    fn create_watcher(workspace: Arc<W>) -> Result<RecommendedWatcher> {
        let watcher = Watcher::new_immediate(move |event: notify::Result<notify::Event>| {
            if let Ok(event) = event {
                if event.kind.is_modify() {
                    for path in event.paths {
                        let _ = workspace.reload(path);
                    }
                }
            }
        })?;
        Ok(watcher)
    }
}

impl<W: Workspace> Workspace for DocumentWatcher<W> {
    fn open(
        &self,
        uri: Arc<Uri>,
        text: String,
        language: DocumentLanguage,
        source: WorkspaceSource,
    ) -> Arc<Document> {
        let document = self.workspace.open(uri, text, language, source);
        if document.uri.scheme() == "file" {
            if let Ok(mut path) = document.uri.to_file_path() {
                path.pop();
                let mut watched_paths = self.watched_paths.lock().unwrap();
                if !watched_paths.contains(&path) {
                    if let Err(why) = self
                        .watcher
                        .lock()
                        .unwrap()
                        .watch(&path, RecursiveMode::NonRecursive)
                    {
                        warn!(
                            "Failed to watch folder of document \"{}\": {}",
                            document.uri, why
                        );
                    }
                    watched_paths.insert(path);
                }
            }
        }
        document
    }

    fn register_open_handler(&self, handler: OpenHandler) {
        self.workspace.register_open_handler(handler);
    }

    fn documents(&self) -> Vec<Arc<Document>> {
        self.workspace.documents()
    }

    fn has(&self, uri: &Uri) -> bool {
        self.workspace.has(uri)
    }

    fn get(&self, uri: &Uri) -> Option<Arc<Document>> {
        self.workspace.get(uri)
    }

    fn close(&self, uri: &Uri) {
        self.workspace.close(uri)
    }

    fn is_open(&self, uri: &Uri) -> bool {
        self.workspace.is_open(uri)
    }

    fn subset(&self, uri: Arc<Uri>) -> Option<WorkspaceSubset> {
        self.workspace.subset(uri)
    }
}
