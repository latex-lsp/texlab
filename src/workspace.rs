use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Result;
use crossbeam_channel::Sender;
use petgraph::{graphmap::UnGraphMap, visit::Dfs};
use rustc_hash::FxHashSet;

use crate::{
    component_db::COMPONENT_DATABASE, Document, DocumentLanguage, DocumentVisibility,
    ServerContext, Uri,
};

#[derive(Debug, Clone)]
pub enum WorkspaceEvent {
    Changed(Workspace, Document),
}

#[derive(Debug, Clone)]
pub struct WorkspaceSubset {
    pub documents: Vec<Document>,
}

#[derive(Debug, Clone, Default)]
pub struct Workspace {
    pub documents_by_uri: im::HashMap<Arc<Uri>, Document>,
    pub listeners: im::Vector<Sender<WorkspaceEvent>>,
}

impl Workspace {
    pub fn open(
        &mut self,
        context: &ServerContext,
        uri: Arc<Uri>,
        text: Arc<String>,
        language: DocumentLanguage,
        visibility: DocumentVisibility,
    ) -> Result<Document> {
        log::debug!("(Re)Loading document: {}", uri);
        let document = Document::parse(context, Arc::clone(&uri), text, language, visibility);

        self.documents_by_uri
            .insert(Arc::clone(&uri), document.clone());

        for listener in &self.listeners {
            listener.send(WorkspaceEvent::Changed(self.clone(), document.clone()))?;
        }

        self.expand_parent(context, &document);
        self.expand_children(context, &document);
        Ok(document)
    }

    pub fn reload(&mut self, context: &ServerContext, path: PathBuf) -> Result<Option<Document>> {
        let uri = Arc::new(Uri::from_file_path(path.clone()).unwrap());

        if self.is_open(&uri) && !uri.as_str().ends_with(".log") {
            return Ok(self.documents_by_uri.get(&uri).cloned());
        }

        if let Some(language) = DocumentLanguage::by_path(&path) {
            let data = fs::read(&path)?;
            let text = Arc::new(String::from_utf8_lossy(&data).into_owned());
            Ok(Some(self.open(
                context,
                uri,
                text,
                language,
                DocumentVisibility::Hidden,
            )?))
        } else {
            Ok(None)
        }
    }

    pub fn load(&mut self, context: &ServerContext, path: PathBuf) -> Result<Option<Document>> {
        let uri = Arc::new(Uri::from_file_path(path.clone()).unwrap());

        if let Some(document) = self.documents_by_uri.get(&uri).cloned() {
            return Ok(Some(document));
        }

        let data = fs::read(&path)?;
        let text = Arc::new(String::from_utf8_lossy(&data).into_owned());
        if let Some(language) = DocumentLanguage::by_path(&path) {
            Ok(Some(self.open(
                context,
                uri,
                text,
                language,
                DocumentVisibility::Hidden,
            )?))
        } else {
            Ok(None)
        }
    }

    pub fn close(&mut self, uri: &Uri) {
        if let Some(document) = self.documents_by_uri.get_mut(uri) {
            document.visibility = DocumentVisibility::Hidden;
        }
    }

    pub fn is_open(&self, uri: &Uri) -> bool {
        self.documents_by_uri.get(uri).map_or(false, |document| {
            document.visibility == DocumentVisibility::Visible
        })
    }

    pub fn subset(&self, uri: Arc<Uri>) -> Option<WorkspaceSubset> {
        let all_current_uris: Vec<Arc<Uri>> = self.documents_by_uri.keys().cloned().collect();

        let mut edges = Vec::new();
        for (i, uri) in all_current_uris.iter().enumerate() {
            let document = self.documents_by_uri.get(uri);
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
            documents.push(self.documents_by_uri.get(&all_current_uris[i]).cloned()?);
        }

        Some(WorkspaceSubset { documents })
    }

    fn has_parent(&self, uri: Arc<Uri>) -> Option<bool> {
        let subset = self.subset(Arc::clone(&uri))?;
        Some(subset.documents.iter().any(|document| {
            document.data.as_latex().map_or(false, |data| {
                data.extras.has_document_environment
                    && !data
                        .extras
                        .explicit_links
                        .iter()
                        .filter_map(|link| link.as_component_name())
                        .any(|name| name == "subfiles.cls")
            })
        }))
    }

    fn expand_parent(&mut self, context: &ServerContext, document: &Document) {
        let all_current_paths = self
            .documents_by_uri
            .values()
            .filter_map(|doc| doc.uri.to_file_path().ok())
            .collect::<FxHashSet<_>>();

        if document.uri.scheme() == "file" {
            if let Ok(mut path) = document.uri.to_file_path() {
                while path.pop() && !self.has_parent(Arc::clone(&document.uri)).unwrap_or(false) {
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
                            let _ = self.load(context, path);
                        });
                }
            }
        }
    }

    fn expand_children(&mut self, context: &ServerContext, document: &Document) {
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
                    if self.load(context, path).is_ok() {
                        break;
                    }
                }
            });
        }
    }
}
