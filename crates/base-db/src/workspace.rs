use std::{borrow::Cow, path::Path};

use rustc_hash::FxHashMap;
use url::Url;

use crate::{Config, Document, DocumentData, Language, Owner};

#[derive(Debug)]
pub struct Workspace {
    documents: FxHashMap<Url, Document>,
    config: Config,
    root_dirs: Vec<Url>,
}

impl Workspace {
    pub fn lookup(&self, uri: &Url) -> Option<&Document> {
        self.documents.get(uri)
    }

    pub fn open(&mut self, uri: Url, text: String, language: Language, owner: Owner) {
        log::debug!("Opening document {uri}...");
        let document = Document::parse(uri, text, language, owner);
        self.documents.insert(document.uri.clone(), document);
    }

    pub fn load(&mut self, path: &Path, language: Language, owner: Owner) -> std::io::Result<()> {
        log::debug!("Loading document {} from disk...", path.display());
        let uri = Url::from_file_path(path).unwrap();
        let data = std::fs::read(path)?;
        let text = match String::from_utf8_lossy(&data) {
            Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(data) },
            Cow::Owned(text) => text,
        };

        Ok(self.open(uri, text, language, owner))
    }

    pub fn watch(&mut self, watcher: &mut dyn notify::Watcher) {
        self.documents
            .values()
            .filter(|document| document.uri.scheme() == "file")
            .flat_map(|document| {
                let dir1 = self.output_dir(&self.current_dir(&document.dir));
                let dir2 = &document.dir;
                [dir1.to_file_path(), dir2.to_file_path()]
            })
            .flatten()
            .for_each(|path| {
                let _ = watcher.watch(&path, notify::RecursiveMode::NonRecursive);
            });
    }

    pub fn current_dir(&self, base_dir: &Url) -> Url {
        let root_dir = self.config.root_dir.as_deref();
        if let Some(dir) = root_dir.and_then(|path| base_dir.join(path).ok()) {
            return dir;
        }

        self.documents
            .values()
            .filter(|doc| matches!(doc.data, DocumentData::Root | DocumentData::Tectonic))
            .flat_map(|doc| doc.uri.join("."))
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
}
