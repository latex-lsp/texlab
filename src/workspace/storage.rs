use std::sync::{Arc, Mutex};

use petgraph::{graphmap::UnGraphMap, visit::Dfs};
use rustc_hash::FxHashMap;

use crate::{
    Document, DocumentLanguage, DocumentVisibility, OpenHandler, ServerContext, Uri, Workspace,
    WorkspaceSubset,
};

#[derive(Clone)]
pub struct Storage {
    context: Arc<ServerContext>,
    documents_by_uri: Arc<Mutex<FxHashMap<Arc<Uri>, Document>>>,
    open_handlers: Arc<Mutex<Vec<OpenHandler>>>,
}

impl Storage {
    pub fn new(context: Arc<ServerContext>) -> Self {
        Self {
            context,
            documents_by_uri: Arc::default(),
            open_handlers: Arc::default(),
        }
    }
}

impl Workspace for Storage {
    fn open(
        &self,
        uri: Arc<Uri>,
        text: Arc<String>,
        language: DocumentLanguage,
        visibility: DocumentVisibility,
    ) -> Document {
        log::debug!("(Re)Loading document: {}", uri);
        let document = Document::parse(
            Arc::clone(&self.context),
            Arc::clone(&uri),
            text,
            language,
            visibility,
        );

        self.documents_by_uri
            .lock()
            .unwrap()
            .insert(Arc::clone(&uri), document.clone());

        let handlers = { self.open_handlers.lock().unwrap().clone() };
        for handler in handlers {
            handler(Arc::new(self.clone()), document.clone());
        }

        document
    }

    fn register_open_handler(&self, handler: OpenHandler) {
        self.open_handlers.lock().unwrap().push(handler);
    }

    fn documents(&self) -> Vec<Document> {
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

    fn get(&self, uri: &Uri) -> Option<Document> {
        self.documents_by_uri.lock().unwrap().get(uri).cloned()
    }

    fn close(&self, uri: &Uri) {
        if let Some(document) = self.documents_by_uri.lock().unwrap().get_mut(uri) {
            document.visibility = DocumentVisibility::Hidden;
        }
    }

    fn delete(&self, uri: &Uri) {
        self.documents_by_uri.lock().unwrap().remove(uri);
    }

    fn is_open(&self, uri: &Uri) -> bool {
        self.documents_by_uri
            .lock()
            .unwrap()
            .get(uri)
            .map_or(false, |document| {
                document.visibility == DocumentVisibility::Visible
            })
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
        let mut dfs = Dfs::new(&graph, start);
        let mut documents = Vec::new();
        while let Some(i) = dfs.next(&graph) {
            documents.push(self.get(&all_current_uris[i])?);
        }

        Some(WorkspaceSubset { documents })
    }
}
