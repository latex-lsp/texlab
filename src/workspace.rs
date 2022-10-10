use std::{
    fs::{self, FileType},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use crossbeam_channel::Sender;
use itertools::Itertools;
use lsp_types::Url;
use notify::Watcher;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{component_db::COMPONENT_DATABASE, Document, DocumentLanguage, Environment};

#[derive(Debug, Clone)]
pub enum WorkspaceEvent {
    Changed(Workspace, Document),
}

#[derive(Debug, Clone, Default)]
pub struct Workspace {
    documents_by_uri: FxHashMap<Arc<Url>, Document>,
    pub viewport: FxHashSet<Arc<Url>>,
    pub listeners: Vec<Sender<WorkspaceEvent>>,
    pub environment: Environment,
    watcher: Option<Arc<Mutex<notify::RecommendedWatcher>>>,
    watched_dirs: Arc<Mutex<FxHashSet<PathBuf>>>,
}

impl Workspace {
    pub fn new(environment: Environment) -> Self {
        Self {
            environment,
            ..Self::default()
        }
    }

    pub fn get(&self, uri: &Url) -> Option<Document> {
        self.documents_by_uri.get(uri).cloned()
    }

    pub fn remove(&mut self, uri: &Url) {
        self.documents_by_uri.remove(uri);
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Document> + 'a {
        self.documents_by_uri.values().cloned()
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
        let mut slice = self.clone();
        slice.documents_by_uri = self
            .get(uri)
            .map(|document| {
                self.siblings(&document)
                    .into_iter()
                    .map(|document| (Arc::clone(document.uri()), document))
                    .collect()
            })
            .unwrap_or_default();
        slice
    }

    fn expand_parent(&mut self, document: &Document) {
        let all_current_paths = self
            .documents_by_uri
            .values()
            .filter_map(|doc| doc.uri().to_file_path().ok())
            .collect::<FxHashSet<_>>();

        if document.uri().scheme() == "file" {
            if let Ok(mut path) = document.uri().to_file_path() {
                while path.pop() && self.parents(&document).next().is_none() {
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
        if let Some(data) = document.data().as_latex() {
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

    pub fn project_roots<'a>(&'a self) -> impl Iterator<Item = Document> + 'a {
        self.iter().filter(|root| {
            root.data()
                .as_latex()
                .map_or(false, |data| data.extras.can_be_root)
        })
    }

    pub fn project_files(&self, root: &Document) -> Vec<Document> {
        let mut results = Vec::new();
        let working_dir = root.uri();
        let mut visited = FxHashSet::default();
        self.visit_project(root, working_dir, &mut visited, &mut results);
        results
    }

    fn visit_project(
        &self,
        root: &Document,
        working_dir: &Url,
        visited: &mut FxHashSet<Arc<Url>>,
        results: &mut Vec<Document>,
    ) {
        if !visited.insert(Arc::clone(root.uri())) {
            return;
        }

        results.push(root.clone());
        if let Some(data) = root.data().as_latex() {
            for link in &data.extras.explicit_links {
                if link
                    .as_component_name()
                    .and_then(|name| COMPONENT_DATABASE.find(&name))
                    .is_some()
                {
                    continue;
                }

                if let Some(child) = link
                    .targets(&working_dir, &self.environment.resolver)
                    .find_map(|uri| self.get(&uri))
                {
                    self.visit_project(&child, &working_dir, visited, results);
                }
            }

            for extension in &["aux", "log"] {
                if let Some(child) = change_extension(root.uri(), extension)
                    .and_then(|file_name| working_dir.join(&file_name).ok())
                    .and_then(|uri| self.get(&uri))
                {
                    self.visit_project(&child, &working_dir, visited, results);
                }
            }
        }
    }

    pub fn parents<'a>(&'a self, child: &'a Document) -> impl Iterator<Item = Document> + 'a {
        self.project_roots().filter(|root| {
            self.project_files(root)
                .iter()
                .any(|doc| doc.uri() == child.uri())
        })
    }

    pub fn working_dir(&self, root: &Document) -> Arc<Url> {
        self.environment
            .options
            .root_directory
            .as_deref()
            .and_then(|path| path.to_str())
            .and_then(|path| root.uri().join(path).map(Arc::new).ok())
            .unwrap_or_else(|| Arc::clone(root.uri()))
    }

    pub fn siblings(&self, child: &Document) -> Vec<Document> {
        self.iter()
            .map(|root| self.project_files(&root))
            .filter(|project| project.iter().any(|doc| doc.uri() == child.uri()))
            .flatten()
            .unique_by(|doc| Arc::clone(doc.uri()))
            .collect()
    }
}

fn change_extension(uri: &Url, extension: &str) -> Option<String> {
    let file_name = uri.path_segments()?.last()?;
    let file_stem = file_name
        .rfind('.')
        .map(|i| &file_name[..i])
        .unwrap_or(file_name);

    Some(format!("{}.{}", file_stem, extension))
}

// fn explore_project(
//     root: &Document,
//     working_dir: &Url,
//     resolver: &Resolver,
//     visited: &mut FxHashSet<Arc<Url>>,
//     results: &mut Vec<Document>,
// ) {
//     if !visited.insert(Arc::clone(root.uri())) {
//         return;
//     }

//     results.push(root.clone());
//     if let Some(data) = root.data().as_latex() {
//         for link in &data.extras.explicit_links {
//             if link
//                 .as_component_name()
//                 .and_then(|name| COMPONENT_DATABASE.find(&name))
//                 .is_some()
//             {
//                 continue;
//             }

//             if let Some(child) = link
//                 .targets(&working_dir, resolver)
//                 .find_map(|uri| self.get(&uri))
//             {
//                 explore_project(&child, &working_dir, visited, results);
//             }
//         }

//         for extension in &["aux", "log"] {
//             if let Some(child) = change_extension(root.uri(), extension)
//                 .and_then(|file_name| working_dir.join(&file_name).ok())
//                 .and_then(|uri| self.get(&uri))
//             {
//                 explore_project(&child, &working_dir, visited, results);
//             }
//         }
//     }
// }
