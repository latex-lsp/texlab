use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Result;
use crossbeam_channel::Sender;
use lsp_types::Url;
use petgraph::{graphmap::UnGraphMap, visit::Dfs};
use rustc_hash::FxHashSet;

use crate::{component_db::COMPONENT_DATABASE, Document, DocumentLanguage, Environment};

#[derive(Debug, Clone)]
pub enum WorkspaceEvent {
    Changed(Workspace, Document),
}

#[derive(Debug, Clone, Default)]
pub struct Workspace {
    pub documents_by_uri: im::HashMap<Arc<Url>, Document>,
    pub viewport: im::HashSet<Arc<Url>>,
    pub listeners: im::Vector<Sender<WorkspaceEvent>>,
    pub environment: Environment,
}

impl Workspace {
    pub fn new(environment: Environment) -> Self {
        Self {
            documents_by_uri: im::HashMap::new(),
            viewport: im::HashSet::new(),
            listeners: im::Vector::new(),
            environment,
        }
    }

    pub fn open(
        &mut self,
        uri: Arc<Url>,
        text: Arc<String>,
        language: DocumentLanguage,
    ) -> Result<Document> {
        log::debug!("(Re)Loading document: {}", uri);
        let document = Document::parse(&self.environment, Arc::clone(&uri), text, language);

        self.documents_by_uri
            .insert(Arc::clone(&uri), document.clone());

        for listener in &self.listeners {
            listener.send(WorkspaceEvent::Changed(self.clone(), document.clone()))?;
        }

        self.expand_parent(&document);
        self.expand_children(&document);
        Ok(document)
    }

    pub fn reload(&mut self, path: PathBuf) -> Result<Option<Document>> {
        let uri = Arc::new(Url::from_file_path(path.clone()).unwrap());

        if self.is_open(&uri) && !uri.as_str().ends_with(".log") {
            return Ok(self.documents_by_uri.get(&uri).cloned());
        }

        if let Some(language) = DocumentLanguage::by_path(&path) {
            let data = fs::read(&path)?;
            let text = Arc::new(String::from_utf8_lossy(&data).into_owned());
            Ok(Some(self.open(uri, text, language)?))
        } else {
            Ok(None)
        }
    }

    pub fn load(&mut self, path: PathBuf) -> Result<Option<Document>> {
        let uri = Arc::new(Url::from_file_path(path.clone()).unwrap());

        if let Some(document) = self.documents_by_uri.get(&uri).cloned() {
            return Ok(Some(document));
        }

        let data = fs::read(&path)?;
        let text = Arc::new(String::from_utf8_lossy(&data).into_owned());
        if let Some(language) = DocumentLanguage::by_path(&path) {
            Ok(Some(self.open(uri, text, language)?))
        } else {
            Ok(None)
        }
    }

    pub fn close(&mut self, uri: &Url) {
        self.viewport.remove(uri);
    }

    pub fn is_open(&self, uri: &Url) -> bool {
        self.viewport.contains(uri)
    }

    pub fn slice(&self, uri: &Url) -> Self {
        let all_uris: Vec<_> = self.documents_by_uri.keys().cloned().collect();

        all_uris
            .iter()
            .position(|u| u.as_ref() == uri)
            .map(|start| {
                let mut edges = Vec::new();
                for (i, uri) in all_uris.iter().enumerate() {
                    let document = self.documents_by_uri.get(uri);
                    if let Some(data) = document
                        .as_ref()
                        .and_then(|document| document.data.as_latex())
                    {
                        let extras = &data.extras;
                        let mut all_targets =
                            vec![&extras.implicit_links.aux, &extras.implicit_links.log];
                        for link in &extras.explicit_links {
                            all_targets.push(&link.targets);
                        }

                        for targets in all_targets {
                            for target in targets {
                                if let Some(j) = all_uris.iter().position(|uri| uri == target) {
                                    edges.push((i, j, ()));
                                    break;
                                }
                            }
                        }
                    }
                }

                let mut slice = self.clone();
                slice.documents_by_uri = im::HashMap::new();
                let graph = UnGraphMap::from_edges(edges);
                let mut dfs = Dfs::new(&graph, start);
                while let Some(i) = dfs.next(&graph) {
                    let uri = &all_uris[i];
                    let doc = self.documents_by_uri[uri].clone();
                    slice.documents_by_uri.insert(Arc::clone(uri), doc);
                }

                slice
            })
            .unwrap_or_default()
    }

    fn find_parent(&self, uri: &Url) -> Option<Document> {
        self.slice(uri)
            .documents_by_uri
            .values()
            .find(|document| {
                document.data.as_latex().map_or(false, |data| {
                    data.extras.has_document_environment
                        && !data
                            .extras
                            .explicit_links
                            .iter()
                            .filter_map(|link| link.as_component_name())
                            .any(|name| name == "subfiles.cls")
                })
            })
            .cloned()
    }

    fn expand_parent(&mut self, document: &Document) {
        let all_current_paths = self
            .documents_by_uri
            .values()
            .filter_map(|doc| doc.uri.to_file_path().ok())
            .collect::<FxHashSet<_>>();

        if document.uri.scheme() == "file" {
            if let Ok(mut path) = document.uri.to_file_path() {
                while path.pop() && self.find_parent(&document.uri).is_none() {
                    std::fs::read_dir(&path)
                        .into_iter()
                        .flatten()
                        .filter_map(|entry| entry.ok())
                        .filter(|entry| entry.file_type().ok().filter(|ty| ty.is_file()).is_some())
                        .map(|entry| entry.path())
                        .filter(|path| {
                            matches!(
                                DocumentLanguage::by_path(path),
                                Some(DocumentLanguage::Latex)
                            )
                        })
                        .filter(|path| !all_current_paths.contains(path))
                        .for_each(|path| {
                            let _ = self.load(path);
                        });
                }
            }
        }
    }

    fn expand_children(&mut self, document: &Document) {
        if let Some(data) = document.data.as_latex() {
            let extras = &data.extras;
            let mut all_targets = vec![&extras.implicit_links.aux, &extras.implicit_links.log];
            for link in &extras.explicit_links {
                if link
                    .as_component_name()
                    .and_then(|name| COMPONENT_DATABASE.find(&name))
                    .is_none()
                {
                    all_targets.push(&link.targets);
                }
            }

            all_targets.into_iter().for_each(|targets| {
                for path in targets
                    .iter()
                    .filter(|uri| uri.scheme() == "file" && uri.fragment().is_none())
                    .filter_map(|uri| uri.to_file_path().ok())
                {
                    if self.load(path).is_ok() {
                        break;
                    }
                }
            });
        }
    }
}
