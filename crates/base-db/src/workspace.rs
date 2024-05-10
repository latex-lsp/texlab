use std::{
    borrow::{Borrow, Cow},
    path::{Path, PathBuf},
};

use distro::{Distro, Language};
use line_index::LineCol;
use rowan::{TextLen, TextRange};
use rustc_hash::{FxHashMap, FxHashSet};
use url::Url;

use crate::{deps, Config, Document, DocumentParams, Owner};

#[derive(Debug, Default)]
pub struct Workspace {
    documents: FxHashSet<Document>,
    config: Config,
    distro: Distro,
    folders: Vec<PathBuf>,
    graphs: FxHashMap<Url, deps::Graph>,
}

impl Workspace {
    pub fn lookup<Q>(&self, key: &Q) -> Option<&Document>
    where
        Q: std::hash::Hash + Eq,
        Document: Borrow<Q>,
    {
        self.documents.get(key)
    }

    pub fn lookup_file(&self, path: &Path) -> Option<&Document> {
        self.iter()
            .find(|document| document.path.as_deref() == Some(path))
    }

    pub fn lookup_file_or_dir<'a>(
        &'a self,
        file_or_dir: &'a Path,
    ) -> impl Iterator<Item = &'a Document> + '_ {
        self.iter().filter(move |doc| {
            doc.path
                .as_deref()
                .map_or(false, |p| p.starts_with(file_or_dir))
        })
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

    pub fn graphs(&self) -> &FxHashMap<Url, deps::Graph> {
        &self.graphs
    }

    pub fn folders(&self) -> &[PathBuf] {
        &self.folders
    }

    pub fn open(
        &mut self,
        uri: Url,
        text: String,
        language: Language,
        owner: Owner,
        cursor: LineCol,
    ) {
        log::debug!("Opening document {uri}...");
        self.documents.remove(&uri);
        self.documents.insert(Document::parse(DocumentParams {
            uri,
            text,
            language,
            owner,
            cursor,
            config: &self.config,
        }));

        self.graphs = self
            .iter()
            .map(|start| (start.uri.clone(), deps::Graph::new(self, start)))
            .collect();
    }

    pub fn load(&mut self, path: &Path, language: Language) -> std::io::Result<()> {
        log::debug!("Loading document {} from disk...", path.display());
        let uri = Url::from_file_path(path).unwrap();
        let data = std::fs::read(path)?;
        let text = match String::from_utf8_lossy(&data) {
            Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(data) },
            Cow::Owned(text) => text,
        };

        let owner = if self.distro.file_name_db.contains(path) {
            Owner::Distro
        } else {
            Owner::Server
        };

        if let Some(document) = self.lookup_file(path) {
            if document.text == text {
                return Ok(());
            }
        }

        self.open(uri, text, language, owner, LineCol { line: 0, col: 0 });
        Ok(())
    }

    pub fn edit(&mut self, uri: &Url, delete: TextRange, insert: &str) -> Option<()> {
        let document = self.lookup(uri)?;
        let mut text = document.text.clone();
        let cursor = if delete.len() == text.text_len() {
            let line = document.cursor.line.min(text.lines().count() as u32);
            LineCol { line, col: 0 }
        } else {
            document.line_index.line_col(delete.start())
        };

        text.replace_range(std::ops::Range::<usize>::from(delete), insert);
        self.open(
            document.uri.clone(),
            text,
            document.language,
            Owner::Client,
            cursor,
        );

        Some(())
    }

    pub fn contains(&self, path: &Path) -> bool {
        if self.folders.is_empty() {
            return true;
        }

        self.folders.iter().any(|dir| path.starts_with(dir))
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

    pub fn set_cursor(&mut self, uri: &Url, cursor: LineCol) -> Option<()> {
        let mut document = self.lookup(uri)?.clone();
        document.cursor = cursor;
        self.documents.remove(&document);
        self.documents.insert(document);
        Some(())
    }

    pub fn reload(&mut self) {
        let uris = self
            .documents
            .iter()
            .filter(|document| document.language == Language::Tex)
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
        log::info!("Removing moved or deleted document: {uri}");
        self.documents.remove(uri);
    }

    pub fn close(&mut self, uri: &Url) -> Option<()> {
        let mut document = self.lookup(uri)?.clone();
        document.owner = Owner::Server;
        self.documents.insert(document);
        Some(())
    }
}
