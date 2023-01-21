use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use lsp_types::{ClientCapabilities, ClientInfo, Url};
use rowan::TextSize;
use rustc_hash::FxHashSet;

use crate::{
    db::document::{Document, Location},
    distro::FileNameDB,
    Db, Options,
};

use super::{
    dependency_graph,
    document::{Contents, Language, LinterData, Owner},
    Word,
};

#[salsa::input(singleton)]
pub struct Workspace {
    #[return_ref]
    pub documents: FxHashSet<Document>,

    #[return_ref]
    pub options: Options,

    #[return_ref]
    pub client_capabilities: ClientCapabilities,

    #[return_ref]
    pub client_info: Option<ClientInfo>,

    #[return_ref]
    pub root_dirs: Vec<Location>,

    #[return_ref]
    pub file_name_db: FileNameDB,
}

impl Workspace {
    pub fn lookup(self, db: &dyn Db, location: Location) -> Option<Document> {
        self.documents(db)
            .iter()
            .find(|document| document.location(db) == location)
            .copied()
    }

    pub fn lookup_uri(self, db: &dyn Db, uri: &Url) -> Option<Document> {
        self.documents(db)
            .iter()
            .find(|document| document.location(db).uri(db) == uri)
            .copied()
    }

    pub fn lookup_path(self, db: &dyn Db, path: &Path) -> Option<Document> {
        self.documents(db)
            .iter()
            .find(|document| document.location(db).path(db).as_deref() == Some(path))
            .copied()
    }

    pub fn index_files<'db>(self, db: &'db dyn Db) -> impl Iterator<Item = Document> + 'db {
        self.documents(db)
            .iter()
            .copied()
            .filter(|&document| document.can_be_index(db))
    }

    pub fn open(
        self,
        db: &mut dyn Db,
        uri: Url,
        text: String,
        language: Language,
        owner: Owner,
    ) -> Document {
        let location = Location::new(db, uri);
        let contents = Contents::new(db, text);
        let cursor = TextSize::from(0);
        match self.lookup(db, location) {
            Some(document) => {
                document.set_contents(db).to(contents);
                document.set_language(db).to(language);
                document.set_owner(db).to(owner);
                document.set_cursor(db).to(cursor);
                document
            }
            None => {
                let document = Document::new(
                    db,
                    location,
                    contents,
                    language,
                    owner,
                    cursor,
                    LinterData::new(db, Vec::new()),
                );

                let mut documents = self.set_documents(db).to(FxHashSet::default());
                documents.insert(document);
                self.set_documents(db).to(documents);
                document
            }
        }
    }

    pub fn load(
        self,
        db: &mut dyn Db,
        path: &Path,
        language: Language,
        owner: Owner,
    ) -> Option<Document> {
        log::debug!("Loading document {} from disk...", path.display());

        let uri = Url::from_file_path(path).ok()?;
        let data = std::fs::read(path).ok()?;
        let text = match String::from_utf8_lossy(&data) {
            Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(data) },
            Cow::Owned(text) => text,
        };

        Some(self.open(db, uri, text, language, owner))
    }

    pub fn watch(
        self,
        db: &dyn Db,
        watcher: &mut dyn notify::Watcher,
        watched_dirs: &mut FxHashSet<PathBuf>,
    ) {
        let output_dirs = self
            .documents(db)
            .iter()
            .map(|document| self.working_dir(db, document.directory(db)))
            .map(|base_dir| self.output_dir(db, base_dir))
            .filter_map(|location| location.path(db).as_deref());

        self.documents(db)
            .iter()
            .map(|document| document.location(db))
            .filter_map(|location| location.path(db).as_deref())
            .filter_map(|path| path.parent())
            .chain(output_dirs)
            .filter(|path| watched_dirs.insert(path.to_path_buf()))
            .for_each(|path| {
                let _ = watcher.watch(path, notify::RecursiveMode::NonRecursive);
            });
    }
}

#[salsa::tracked]
impl Workspace {
    #[salsa::tracked]
    pub fn working_dir(self, db: &dyn Db, base_dir: Location) -> Location {
        if let Some(dir) = self
            .options(db)
            .root_directory
            .as_deref()
            .and_then(|path| path.to_str())
            .and_then(|path| base_dir.join(db, path))
        {
            return dir;
        }

        self.documents(db)
            .iter()
            .filter(|doc| matches!(doc.language(db), Language::TexlabRoot | Language::Tectonic))
            .filter_map(|doc| doc.location(db).join(db, "."))
            .find(|root_dir| {
                base_dir
                    .uri(db)
                    .as_str()
                    .starts_with(root_dir.uri(db).as_str())
            })
            .unwrap_or(base_dir)
    }

    #[salsa::tracked]
    pub fn output_dir(self, db: &dyn Db, base_dir: Location) -> Location {
        let mut path = self
            .options(db)
            .aux_directory
            .as_deref()
            .and_then(|path| path.to_str())
            .unwrap_or(".")
            .to_string();

        if !path.ends_with("/") {
            path.push('/');
        }

        base_dir.join(db, &path).unwrap_or(base_dir)
    }

    #[salsa::tracked(return_ref)]
    pub fn parents(self, db: &dyn Db, child: Document) -> Vec<Document> {
        self.index_files(db)
            .filter(|&parent| dependency_graph(db, parent).preorder().contains(&child))
            .collect()
    }

    #[salsa::tracked(return_ref)]
    pub fn related(self, db: &dyn Db, child: Document) -> FxHashSet<Document> {
        self.index_files(db)
            .chain(self.documents(db).iter().copied())
            .map(|start| dependency_graph(db, start).preorder().collect_vec())
            .filter(|project| project.contains(&child))
            .flatten()
            .collect()
    }

    #[salsa::tracked]
    pub fn number_of_label(self, db: &dyn Db, child: Document, name: Word) -> Option<Word> {
        self.related(db, child)
            .iter()
            .filter_map(|document| document.parse(db).as_tex())
            .flat_map(|data| data.analyze(db).label_numbers(db))
            .find(|number| number.name(db) == name)
            .map(|number| number.text(db))
    }
}
