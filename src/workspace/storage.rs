use std::sync::{Arc, Mutex};

use petgraph::{graphmap::UnGraphMap, visit::Dfs};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    Document, DocumentLanguage, OpenHandler, ServerContext, Uri, Workspace, WorkspaceSource,
    WorkspaceSubset,
};

pub struct Storage {
    context: Arc<ServerContext>,
    documents_by_uri: Mutex<FxHashMap<Arc<Uri>, Arc<Document>>>,
    opened_documents: Mutex<FxHashSet<Arc<Uri>>>,
    open_handlers: Mutex<Vec<OpenHandler>>,
}

impl Storage {
    pub fn new(context: Arc<ServerContext>) -> Self {
        Self {
            context,
            documents_by_uri: Mutex::default(),
            opened_documents: Mutex::default(),
            open_handlers: Mutex::default(),
        }
    }
}

impl Workspace for Storage {
    fn open(
        &self,
        uri: Arc<Uri>,
        text: String,
        language: DocumentLanguage,
        source: WorkspaceSource,
    ) -> Arc<Document> {
        log::debug!("(Re)Loading document: {}", uri);
        let document = Arc::new(Document::parse(
            Arc::clone(&self.context),
            Arc::clone(&uri),
            text,
            language,
        ));
        {
            self.documents_by_uri
                .lock()
                .unwrap()
                .insert(Arc::clone(&uri), Arc::clone(&document));
        }

        if source == WorkspaceSource::Client {
            self.opened_documents.lock().unwrap().insert(uri);
        }

        let handlers = { self.open_handlers.lock().unwrap().clone() };
        for handler in handlers {
            handler(Arc::clone(&document));
        }

        document
    }

    fn register_open_handler(&self, handler: OpenHandler) {
        self.open_handlers.lock().unwrap().push(handler);
    }

    fn documents(&self) -> Vec<Arc<Document>> {
        self.documents_by_uri
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    fn has(&self, uri: &Uri) -> bool {
        self.documents_by_uri.lock().unwrap().contains_key(uri)
    }

    fn get(&self, uri: &Uri) -> Option<Arc<Document>> {
        self.documents_by_uri.lock().unwrap().get(uri).cloned()
    }

    fn close(&self, uri: &Uri) {
        self.opened_documents.lock().unwrap().remove(uri);
    }

    fn is_open(&self, uri: &Uri) -> bool {
        self.opened_documents.lock().unwrap().contains(uri)
    }

    fn subset(&self, uri: Arc<Uri>) -> Option<WorkspaceSubset> {
        let all_current_uris: Vec<Arc<Uri>> = self
            .documents_by_uri
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .collect();

        let mut edges = Vec::new();
        for (i, uri) in all_current_uris.iter().enumerate() {
            let document = self.get(uri);
            if let Some(data) = document
                .as_ref()
                .and_then(|document| document.data.as_latex())
            {
                let extras = &data.extras;
                let mut all_targets = vec![&extras.implicit_links.aux, &extras.implicit_links.log];
                for link in &extras.explicit_links {
                    all_targets.push(&link.targets);
                }

                for targets in all_targets {
                    for target in targets {
                        if let Some(j) = all_current_uris.iter().position(|uri| uri == target) {
                            edges.push((i, j, ()));
                            break;
                        }
                    }
                }
            }
        }

        let graph = UnGraphMap::from_edges(edges);
        let start = all_current_uris.iter().position(|u| *u == uri)?;
        let mut dfs = Dfs::new(&graph, start.into());
        let mut documents = Vec::new();
        while let Some(i) = dfs.next(&graph) {
            documents.push(self.get(&all_current_uris[i])?);
        }

        Some(WorkspaceSubset { documents })
    }
}
