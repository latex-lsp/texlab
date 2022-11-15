use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use lsp_types::{ClientCapabilities, ClientInfo, Url};
use once_cell::sync::Lazy;
use rowan::TextSize;
use rustc_hash::FxHashSet;

use crate::{
    db::document::{Document, Location},
    distro::Resolver,
    Db, Options,
};

use super::{
    analysis::TexLink,
    dependency,
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
    pub file_name_db: Resolver,
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
            .map(|document| self.working_dir(db, document.location(db)))
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

    pub fn discover(self, db: &mut dyn Db) {
        loop {
            let mut changed = false;

            let dirs: FxHashSet<PathBuf> = self
                .documents(db)
                .iter()
                .filter_map(|document| document.location(db).path(db).as_deref())
                .filter_map(|path| path.parent())
                .flat_map(|path| path.ancestors())
                .map(ToOwned::to_owned)
                .collect();

            for dir in dirs {
                let files = std::fs::read_dir(dir)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter(|entry| entry.file_type().map_or(false, |ty| ty.is_file()))
                    .map(|entry| entry.path())
                    .filter(|path| Language::from_path(&path) == Some(Language::Tex));

                for file in files {
                    if self.lookup_path(db, &file).is_none() {
                        changed |= self.load(db, &file, Language::Tex, Owner::Server).is_some();
                    }
                }
            }

            let paths: FxHashSet<_> = self
                .documents(db)
                .iter()
                .map(|&document| self.graph(db, document))
                .flat_map(|graph| graph.edges(db).iter())
                .flat_map(|item| item.origin(db).into_locations(db, self))
                .filter_map(|location| location.path(db).as_deref())
                .filter(|path| path.is_file())
                .map(ToOwned::to_owned)
                .collect();

            for path in paths {
                if self.lookup_path(db, &path).is_none() {
                    let language = Language::from_path(&path).unwrap_or(Language::Tex);
                    changed |= self.load(db, &path, language, Owner::Server).is_some();
                }
            }

            if !changed {
                break;
            }
        }
    }
}

#[salsa::tracked]
impl Workspace {
    #[salsa::tracked]
    pub fn working_dir(self, db: &dyn Db, base_dir: Location) -> Location {
        let path = self
            .options(db)
            .root_directory
            .as_deref()
            .and_then(|path| path.to_str())
            .unwrap_or(".");

        base_dir.join(db, path).unwrap_or(base_dir)
    }

    #[salsa::tracked]
    pub fn output_dir(self, db: &dyn Db, base_dir: Location) -> Location {
        let path = self
            .options(db)
            .aux_directory
            .as_deref()
            .and_then(|path| path.to_str())
            .unwrap_or(".");

        base_dir.join(db, path).unwrap_or(base_dir)
    }

    #[salsa::tracked]
    pub fn graph(self, db: &dyn Db, document: Document) -> dependency::Graph {
        let base_dir = self.working_dir(db, document.location(db));
        let mut items = Vec::new();
        let mut stack = vec![(document, base_dir)];
        let mut visited = FxHashSet::default();

        while let Some((source, dir)) = stack.pop() {
            for item in self
                .explicit_links(db, source, dir)
                .as_deref()
                .unwrap_or_default()
            {
                let link = item.origin(db).into_explicit().unwrap().link;

                if let Some(target) = item.target(db) {
                    if visited.insert(target) {
                        let new_dir = link
                            .working_dir(db)
                            .and_then(|path| dir.join(db, &path.text(db)))
                            .unwrap_or(dir);

                        stack.push((target, new_dir));
                    }
                }

                items.push(*item);
            }

            let output_dir = self.output_dir(db, dir);
            items.extend(self.implicit_links(db, source, output_dir, "aux"));
            items.extend(self.implicit_links(db, source, output_dir, "log"));
        }

        dependency::Graph::new(db, items, document)
    }

    #[salsa::tracked(return_ref)]
    pub fn link_locations(self, db: &dyn Db, link: TexLink, base_dir: Location) -> Vec<Location> {
        let stem = link.path(db).text(db);
        let paths = link
            .kind(db)
            .extensions()
            .iter()
            .map(|ext| format!("{stem}.{ext}"));

        let file_name_db = self.file_name_db(db);
        let distro_files = std::iter::once(stem.to_string())
            .chain(paths.clone())
            .filter_map(|path| file_name_db.get(path.as_str()))
            .filter(|path| {
                HOME_DIR
                    .as_deref()
                    .map_or(false, |dir| path.starts_with(dir))
            })
            .flat_map(|path| Url::from_file_path(path))
            .map(|uri| Location::new(db, uri));

        std::iter::once(stem.to_string())
            .chain(paths)
            .flat_map(|path| base_dir.uri(db).join(&path))
            .map(|uri| Location::new(db, uri))
            .chain(distro_files)
            .collect()
    }

    #[salsa::tracked(return_ref)]
    pub fn explicit_links(
        self,
        db: &dyn Db,
        document: Document,
        base_dir: Location,
    ) -> Option<Vec<dependency::Resolved>> {
        let data = document.parse(db).as_tex()?;

        let mut items = Vec::new();
        for link in data.analyze(db).links(db).iter().copied() {
            let origin = dependency::Origin::Explicit(dependency::Explicit { link, base_dir });
            let target = self
                .link_locations(db, link, base_dir)
                .iter()
                .find_map(|location| self.lookup(db, *location));

            items.push(dependency::Resolved::new(db, document, target, origin));
        }

        Some(items)
    }

    #[salsa::tracked]
    pub fn implicit_links(
        self,
        db: &dyn Db,
        document: Document,
        base_dir: Location,
        extension: &'static str,
    ) -> Option<dependency::Resolved> {
        let stem = document.location(db).stem(db)?;
        let name = format!("{stem}.{extension}");
        let location = self.output_dir(db, base_dir).join(db, &name)?;
        let target = self.lookup(db, location);
        let origin = dependency::Origin::Implicit(dependency::Implicit::new(db, vec![location]));
        Some(dependency::Resolved::new(db, document, target, origin))
    }

    #[salsa::tracked(return_ref)]
    pub fn parents(self, db: &dyn Db, child: Document) -> Vec<Document> {
        self.index_files(db)
            .filter(|&document| self.graph(db, document).preorder(db).contains(&child))
            .collect()
    }

    #[salsa::tracked(return_ref)]
    pub fn related(self, db: &dyn Db, child: Document) -> FxHashSet<Document> {
        self.index_files(db)
            .chain(self.documents(db).iter().copied())
            .map(|document| self.graph(db, document).preorder(db))
            .filter(|project| project.contains(&child))
            .flatten()
            .copied()
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

static HOME_DIR: Lazy<Option<PathBuf>> = Lazy::new(|| dirs::home_dir());
