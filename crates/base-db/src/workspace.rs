use std::{
    borrow::{Borrow, Cow},
    path::{Path, PathBuf},
};

use distro::{Distro, Language};
use itertools::Itertools;
use rowan::{TextRange, TextSize};
use rustc_hash::FxHashSet;
use url::Url;

use crate::{graph, Config, Document, DocumentData, Owner};

#[derive(Debug, Default)]
pub struct Workspace {
    documents: FxHashSet<Document>,
    config: Config,
    distro: Distro,
    folders: Vec<PathBuf>,
}

impl Workspace {
    pub fn lookup<Q>(&self, key: &Q) -> Option<&Document>
    where
        Q: std::hash::Hash + Eq,
        Document: Borrow<Q>,
    {
        self.documents.get(key)
    }

    pub fn lookup_path(&self, path: &Path) -> Option<&Document> {
        self.iter()
            .find(|document| document.path.as_deref() == Some(path))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Document> + '_ {
        self.documents.iter()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn distro(&self) -> &Distro {
        &self.distro
    }

    pub fn open(
        &mut self,
        uri: Url,
        text: String,
        language: Language,
        owner: Owner,
        cursor: TextSize,
    ) {
        log::debug!("Opening document {uri}...");
        self.documents.remove(&uri);
        self.documents.insert(Document::parse(
            uri,
            text,
            language,
            owner,
            cursor,
            &self.config,
        ));
    }

    pub fn load(&mut self, path: &Path, language: Language, owner: Owner) -> std::io::Result<()> {
        log::debug!("Loading document {} from disk...", path.display());
        let uri = Url::from_file_path(path).unwrap();
        let data = std::fs::read(path)?;
        let text = match String::from_utf8_lossy(&data) {
            Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(data) },
            Cow::Owned(text) => text,
        };

        Ok(self.open(uri, text, language, owner, TextSize::default()))
    }

    pub fn edit(&mut self, uri: &Url, delete: TextRange, insert: &str) -> Option<()> {
        let document = self.lookup(uri)?;
        let mut text = document.text.clone();
        text.replace_range(std::ops::Range::<usize>::from(delete), insert);
        self.open(
            document.uri.clone(),
            text,
            document.language,
            Owner::Client,
            delete.start(),
        );

        Some(())
    }

    pub fn watch(
        &mut self,
        watcher: &mut dyn notify::Watcher,
        watched_dirs: &mut FxHashSet<PathBuf>,
    ) {
        self.iter()
            .filter(|document| document.uri.scheme() == "file")
            .flat_map(|document| {
                let dir1 = self.output_dir(&self.current_dir(&document.dir));
                let dir2 = &document.dir;
                [dir1.to_file_path(), dir2.to_file_path()]
            })
            .flatten()
            .for_each(|path| {
                if !watched_dirs.contains(&path) {
                    let _ = watcher.watch(&path, notify::RecursiveMode::NonRecursive);
                    watched_dirs.insert(path);
                }
            });
    }

    pub fn current_dir(&self, base_dir: &Url) -> Url {
        let root_dir = self.config.root_dir.as_deref();
        if let Some(dir) = root_dir.and_then(|path| base_dir.join(path).ok()) {
            return dir;
        }

        self.iter()
            .filter(|document| matches!(document.data, DocumentData::Root | DocumentData::Tectonic))
            .flat_map(|document| document.uri.join("."))
            .find(|root_dir| base_dir.as_str().starts_with(root_dir.as_str()))
            .unwrap_or_else(|| base_dir.clone())
    }

    pub fn output_dir(&self, base_dir: &Url) -> Url {
        let mut path = self.config.build.output_dir.clone();
        if !path.ends_with('/') {
            path.push('/');
        }

        base_dir.join(&path).unwrap_or_else(|_| base_dir.clone())
    }

    pub fn contains(&self, path: &Path) -> bool {
        if self.folders.is_empty() {
            return true;
        }

        self.folders.iter().any(|dir| path.starts_with(dir))
    }

    pub fn related(&self, child: &Document) -> FxHashSet<&Document> {
        let mut results = FxHashSet::default();
        for graph in self
            .iter()
            .map(|start| graph::Graph::new(self, start))
            .filter(|graph| graph.preorder().contains(&child))
        {
            results.extend(graph.preorder());
        }

        results
    }

    pub fn parents(&self, child: &Document) -> FxHashSet<&Document> {
        self.iter()
            .filter(|document| {
                let DocumentData::Tex(data) = &document.data else { return false };
                data.semantics.can_be_root
            })
            .filter(|parent| {
                let graph = graph::Graph::new(self, parent);
                let mut nodes = graph.preorder();
                nodes.contains(&child)
            })
            .collect()
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
        self.reload();
    }

    pub fn set_distro(&mut self, distro: Distro) {
        self.distro = distro;
        self.reload();
    }

    pub fn set_folders(&mut self, folders: Vec<PathBuf>) {
        self.folders = folders;
    }

    pub fn set_cursor(&mut self, uri: &Url, cursor: TextSize) -> Option<()> {
        let mut document = self.lookup(uri)?.clone();
        document.cursor = cursor;
        self.documents.insert(document);
        Some(())
    }

    pub fn reload(&mut self) {
        let uris = self
            .documents
            .iter()
            .map(|document| document.uri.clone())
            .collect::<Vec<Url>>();

        for uri in uris {
            let document = self.lookup(&uri).unwrap();
            self.open(
                uri,
                document.text.clone(),
                document.language,
                document.owner,
                document.cursor,
            );
        }
    }

    pub fn remove(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }

    pub fn close(&mut self, uri: &Url) -> Option<()> {
        let mut document = self.lookup(uri)?.clone();
        document.owner = Owner::Server;
        self.documents.insert(document);
        Some(())
    }

    pub fn discover(&mut self) {
        loop {
            let mut changed = false;
            changed |= self.discover_parents();
            changed |= self.discover_children();
            if !changed {
                break;
            }
        }
    }

    fn discover_parents(&mut self) -> bool {
        let dirs = self
            .iter()
            .filter_map(|document| document.path.as_deref())
            .flat_map(|path| path.ancestors().skip(1))
            .filter(|path| self.contains(path))
            .map(|path| path.to_path_buf())
            .collect::<FxHashSet<_>>();

        let mut changed = false;
        for dir in dirs {
            if self
                .iter()
                .filter(|document| matches!(document.language, Language::Root | Language::Tectonic))
                .filter_map(|document| document.path.as_deref())
                .filter_map(|path| path.parent())
                .any(|marker| dir.starts_with(marker))
            {
                continue;
            }

            let Ok(entries) = std::fs::read_dir(dir) else { continue };

            for file in entries
                .flatten()
                .filter(|entry| entry.file_type().map_or(false, |type_| type_.is_file()))
                .map(|entry| entry.path())
            {
                let Some(lang) = Language::from_path(&file) else { continue };
                if !matches!(lang, Language::Tex | Language::Root | Language::Tectonic) {
                    continue;
                }

                if self.lookup_path(&file).is_none() {
                    changed |= self.load(&file, lang, Owner::Server).is_ok();
                }
            }
        }

        changed
    }

    fn discover_children(&mut self) -> bool {
        let paths = self
            .iter()
            .map(|start| graph::Graph::new(self, start))
            .flat_map(|graph| graph.missing)
            .filter(|uri| uri.scheme() == "file")
            .flat_map(|uri| uri.to_file_path())
            .collect::<FxHashSet<_>>();

        let mut changed = false;
        for path in paths {
            let language = Language::from_path(&path).unwrap_or(Language::Tex);
            if self.lookup_path(&path).is_none() {
                changed |= self.load(&path, language, Owner::Server).is_ok();
            }
        }

        changed
    }
}
