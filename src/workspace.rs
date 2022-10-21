use std::{
    fs::{self, FileType},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use crossbeam_channel::Sender;
use lsp_types::Url;
use notify::Watcher;
use once_cell::sync::Lazy;
use petgraph::{graphmap::DiGraphMap, visit::Dfs};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    component_db::COMPONENT_DATABASE, distro::Resolver, syntax::latex::ExplicitLink, Document,
    DocumentLanguage, Environment,
};

#[derive(Debug, Clone)]
pub enum WorkspaceEvent {
    Changed(Workspace, Document),
}

#[derive(Debug, Clone, Default)]
pub struct Workspace {
    pub documents_by_uri: FxHashMap<Arc<Url>, Document>,
    pub viewport: FxHashSet<Arc<Url>>,
    pub listeners: Vec<Sender<WorkspaceEvent>>,
    pub environment: Environment,
    watcher: Option<Arc<Mutex<notify::RecommendedWatcher>>>,
    watched_dirs: Arc<Mutex<FxHashSet<PathBuf>>>,
}

impl Workspace {
    #[must_use]
    pub fn new(environment: Environment) -> Self {
        Self {
            environment,
            ..Self::default()
        }
    }

    pub fn register_watcher(&mut self, watcher: notify::RecommendedWatcher) {
        self.watcher = Some(Arc::new(Mutex::new(watcher)));
    }

    pub fn watch_dir(&self, path: &Path) {
        if let Some(watcher) = &self.watcher {
            if self.watched_dirs.lock().unwrap().insert(path.to_owned()) {
                let _ = watcher
                    .lock()
                    .unwrap()
                    .watch(path, notify::RecursiveMode::NonRecursive);
            }
        }
    }

    pub fn open(
        &mut self,
        uri: Arc<Url>,
        text: Arc<String>,
        language: DocumentLanguage,
    ) -> Result<Document> {
        if uri.scheme() == "file" {
            if let Ok(mut path) = uri.to_file_path() {
                path.pop();
                self.watch_dir(&path);
            }
        }

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
        if self.is_open(&uri) || !(uri.as_str().ends_with(".log") || uri.as_str().ends_with(".aux"))
        {
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

                                    if target.as_str().ends_with(".tex")
                                        || target.as_str().ends_with(".bib")
                                        || target.as_str().ends_with(".rnw")
                                    {
                                        edges.push((j, i, ()));
                                    }

                                    break;
                                }
                            }
                        }
                    }
                }

                let mut slice = self.clone();
                slice.documents_by_uri = FxHashMap::default();
                let graph = DiGraphMap::from_edges(edges);
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

    #[must_use]
    pub fn find_parent(&self, uri: &Url) -> Option<Document> {
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
                            .filter_map(ExplicitLink::as_component_name)
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
                        .filter_map(Result::ok)
                        .filter(|entry| entry.file_type().ok().filter(FileType::is_file).is_some())
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
                if should_follow_link(link, &self.environment.resolver) {
                    all_targets.push(&link.targets);
                }
            }

            for targets in all_targets {
                for path in targets
                    .iter()
                    .filter(|uri| uri.scheme() == "file" && uri.fragment().is_none())
                    .filter_map(|uri| uri.to_file_path().ok())
                {
                    if self.load(path).is_ok() {
                        break;
                    }
                }
            }
        }
    }
}

static HOME_DIR: Lazy<Option<PathBuf>> = Lazy::new(|| dirs::home_dir());

fn should_follow_link(link: &ExplicitLink, resolver: &Resolver) -> bool {
    match link.as_component_name() {
        Some(name) if COMPONENT_DATABASE.find(&name).is_some() => false,
        Some(name) => {
            let file = resolver.files_by_name.get(name.as_str());
            let home = HOME_DIR.as_deref();
            match (file, home) {
                (Some(file), Some(home)) => file.starts_with(home),
                (Some(_), None) => false,
                (None, Some(_)) => true,
                (None, None) => true,
            }
        }
        None => true,
    }
}
