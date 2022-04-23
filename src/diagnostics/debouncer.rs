use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crossbeam_channel::Sender;
use dashmap::DashMap;
use lsp_types::Url;

use crate::{Document, Workspace};

pub enum DiagnosticsMessage {
    Analyze {
        workspace: Workspace,
        document: Document,
    },
    Shutdown,
}

pub struct DiagnosticsDebouncer {
    pub sender: Sender<DiagnosticsMessage>,
    handle: Option<JoinHandle<()>>,
}

impl DiagnosticsDebouncer {
    pub fn launch<A>(action: A) -> Self
    where
        A: Fn(Workspace, Document) + Send + Clone + 'static,
    {
        let (sender, receiver) = crossbeam_channel::unbounded();

        let handle = thread::spawn(move || {
            let pool = threadpool::Builder::new().build();
            let last_task_time_by_uri: Arc<DashMap<Arc<Url>, Instant>> = Arc::default();
            while let Ok(DiagnosticsMessage::Analyze {
                workspace,
                document,
            }) = receiver.recv()
            {
                let delay = workspace
                    .environment
                    .options
                    .diagnostics_delay
                    .unwrap_or(300);

                if let Some(time) = last_task_time_by_uri.get(&document.uri) {
                    if time.elapsed().as_millis() < delay as u128 {
                        continue;
                    }
                }

                let last_task_time_by_uri = Arc::clone(&last_task_time_by_uri);
                let action = action.clone();
                pool.execute(move || {
                    thread::sleep(Duration::from_millis(delay));
                    last_task_time_by_uri.insert(Arc::clone(&document.uri), Instant::now());
                    action(workspace, document);
                });
            }
        });

        Self {
            sender,
            handle: Some(handle),
        }
    }
}

impl Drop for DiagnosticsDebouncer {
    fn drop(&mut self) {
        self.sender.send(DiagnosticsMessage::Shutdown).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }
}
