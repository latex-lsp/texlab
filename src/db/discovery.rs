use std::path::Path;

use itertools::Itertools;
use lsp_types::Url;
use rustc_hash::FxHashSet;

use crate::{util::HOME_DIR, Db};

use super::{
    analysis::TexLink,
    document::{Document, Language, Location, Owner},
    workspace::Workspace,
};

#[salsa::accumulator]
pub struct MissingDependencies(MissingDependency);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct MissingDependency {
    pub location: Location,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Dependency {
    pub source: Document,
    pub target: Document,
    pub origin: Option<DependencyOrigin>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct DependencyOrigin {
    pub link: TexLink,
    pub old_base_dir: Location,
    pub new_base_dir: Location,
}

pub fn hidden_dependencies(
    db: &dyn Db,
    document: Document,
    base_dir: Location,
    dependencies: &mut Vec<Dependency>,
) {
    let uri = document.location(db).uri(db).as_str();
    if document.language(db) == Language::Tex && !uri.ends_with(".aux") {
        dependencies.extend(hidden_dependency(db, document, base_dir, "log"));
        dependencies.extend(hidden_dependency(db, document, base_dir, "aux"));
    }
}

#[salsa::tracked]
pub fn hidden_dependency(
    db: &dyn Db,
    source: Document,
    base_dir: Location,
    extension: &'static str,
) -> Option<Dependency> {
    let workspace = Workspace::get(db);

    let stem = source.location(db).stem(db)?;
    let name = format!("{stem}.{extension}");

    let location = workspace.output_dir(db, base_dir).join(db, &name)?;
    match workspace.lookup(db, location) {
        Some(target) => Some(Dependency {
            source,
            target,
            origin: None,
        }),
        None => {
            MissingDependencies::push(db, MissingDependency { location });
            None
        }
    }
}

pub fn source_dependencies<'db>(
    db: &'db dyn Db,
    source: Document,
    base_dir: Location,
) -> impl Iterator<Item = Dependency> + 'db {
    source
        .parse(db)
        .as_tex()
        .into_iter()
        .flat_map(|data| data.analyze(db).links(db))
        .filter_map(move |link| source_dependency(db, source, base_dir, *link))
}

#[salsa::tracked]
pub fn source_dependency(
    db: &dyn Db,
    source: Document,
    base_dir: Location,
    link: TexLink,
) -> Option<Dependency> {
    let workspace = Workspace::get(db);

    let stem = link.path(db).text(db);

    let mut file_names = vec![stem.clone()];
    link.kind(db)
        .extensions()
        .iter()
        .map(|ext| format!("{stem}.{ext}"))
        .for_each(|name| file_names.push(name));

    let file_name_db = workspace.file_name_db(db);
    let distro_files = file_names
        .iter()
        .filter_map(|name| file_name_db.get(name))
        .filter(|path| {
            HOME_DIR
                .as_deref()
                .map_or(false, |dir| path.starts_with(dir))
        })
        .flat_map(Url::from_file_path)
        .map(|uri| Location::new(db, uri));

    for location in file_names
        .iter()
        .filter_map(|file_name| base_dir.join(db, file_name))
        .chain(distro_files)
    {
        match workspace.lookup(db, location) {
            Some(target) => {
                let origin = Some(DependencyOrigin {
                    link,
                    old_base_dir: base_dir,
                    new_base_dir: link
                        .base_dir(db)
                        .and_then(|path| base_dir.join(db, path.text(db)))
                        .unwrap_or(base_dir),
                });

                return Some(Dependency {
                    source,
                    target,
                    origin,
                });
            }
            None => {
                MissingDependencies::push(db, MissingDependency { location });
            }
        };
    }

    None
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DependencyGraph {
    pub start: Document,
    pub edges: Vec<Dependency>,
}

impl DependencyGraph {
    pub fn preorder(&self) -> impl DoubleEndedIterator<Item = Document> + '_ {
        std::iter::once(self.start)
            .chain(self.edges.iter().map(|group| group.target))
            .unique()
    }
}

#[salsa::tracked(return_ref)]
pub fn dependency_graph(db: &dyn Db, start: Document) -> DependencyGraph {
    let workspace = Workspace::get(db);

    let base_dir = workspace.working_dir(db, start.directory(db));
    let mut edges = Vec::new();
    let mut stack = vec![(start, base_dir)];
    let mut visited = FxHashSet::default();

    while let Some((source, base_dir)) = stack.pop() {
        for edge in source_dependencies(db, source, base_dir) {
            edges.push(edge);
            if visited.insert(edge.target) {
                stack.push((edge.target, edge.origin.unwrap().new_base_dir));
            }
        }

        hidden_dependencies(db, source, base_dir, &mut edges);
    }

    DependencyGraph { start, edges }
}

pub fn discover_dependencies(db: &mut dyn Db) {
    let workspace = Workspace::get(db);
    loop {
        let mut changed = discover_parents(db, workspace);

        let paths: FxHashSet<_> = workspace
            .documents(db)
            .iter()
            .flat_map(|&start| dependency_graph::accumulated::<MissingDependencies>(db, start))
            .filter_map(|link| link.location.path(db).as_deref())
            .filter(|path| path.is_file())
            .map(Path::to_path_buf)
            .collect();

        for path in paths {
            if workspace.lookup_path(db, &path).is_none() {
                let language = Language::from_path(&path).unwrap_or(Language::Tex);
                changed |= workspace.load(db, &path, language, Owner::Server).is_some();
            }
        }

        if !changed {
            break;
        }
    }
}

fn discover_parents(db: &mut dyn Db, workspace: Workspace) -> bool {
    let mut changed = false;

    let dirs: FxHashSet<_> = workspace
        .documents(db)
        .iter()
        .flat_map(|document| document.ancestor_dirs(db))
        .filter(|path| is_part_of_workspace(db, workspace, path))
        .map(Path::to_path_buf)
        .collect();

    for path in dirs
        .iter()
        .flat_map(std::fs::read_dir)
        .flatten()
        .flatten()
        .filter(|entry| entry.file_type().map_or(false, |ty| ty.is_file()))
        .map(|entry| entry.path())
    {
        if let Some(language) = Language::from_path(&path) {
            let can_be_parent = matches!(
                language,
                Language::Tex | Language::TexlabRoot | Language::Tectonic
            );

            if can_be_parent && workspace.lookup_path(db, &path).is_none() {
                changed |= workspace.load(db, &path, language, Owner::Server).is_some();
            }
        }
    }

    changed
}

fn is_part_of_workspace(db: &dyn Db, workspace: Workspace, path: &Path) -> bool {
    let root_dirs = workspace.root_dirs(db);
    if root_dirs.is_empty() {
        return true;
    }

    root_dirs
        .iter()
        .filter_map(|dir| dir.path(db).as_deref())
        .any(|dir| path.starts_with(dir))
}
