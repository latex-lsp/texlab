use itertools::Itertools;
use lsp_types::Url;
use rustc_hash::FxHashSet;

use crate::{
    component_db::COMPONENT_DATABASE,
    db::{DocumentData, FileId, Language, ServerContext},
    distro::Resolver,
    Db,
};

use super::Document;

#[salsa::interned]
pub struct Dependency {
    pub document: Document,
    pub base_dir: FileId,
}

#[salsa::tracked(return_ref)]
pub fn implicit_dependencies_of(db: &dyn Db, parent: Dependency) -> Vec<Dependency> {
    let context = ServerContext::get(db);

    let mut results = Vec::new();
    for extension in &["aux", "log"] {
        let file_name = parent
            .document(db)
            .file(db)
            .stem(db)
            .as_deref()
            .map(|stem| format!("{}.{}", stem, extension));

        let child = file_name
            .and_then(|name| {
                parent
                    .base_dir(db)
                    .join(db, context.artifact_dir(db))
                    .and_then(|file| file.join(db, &name))
                    .ok()
            })
            .and_then(|file| db.workspace().load(db, file));

        if let Some(child) = child {
            let artifact = Dependency::new(db, child, parent.base_dir(db));
            results.push(artifact);
        }
    }

    results
}

#[salsa::tracked(return_ref)]
pub fn explicit_dependencies_of(db: &dyn Db, parent: Dependency) -> Vec<Dependency> {
    let resolver = Resolver::default();
    let mut results = Vec::new();
    let data = match parent.document(db).parse(db) {
        DocumentData::Tex(data) => data,
        _ => return results,
    };

    let extras = data.extras(db);
    for link in &extras.explicit_links {
        if link
            .as_component_name()
            .and_then(|name| COMPONENT_DATABASE.find(&name))
            .is_some()
        {
            continue;
        }

        let mut targets = link
            .targets(parent.base_dir(db).uri(db), &resolver)
            .map(|uri| FileId::new(db, uri));

        if let Some(child) = targets.find_map(|file| db.workspace().load(db, file)) {
            let base_dir = link
                .working_dir
                .as_ref()
                .and_then(|path| parent.base_dir(db).join(db, path).ok())
                .unwrap_or_else(|| parent.base_dir(db));

            results.push(Dependency::new(db, child, base_dir));
        }
    }

    results
}

#[salsa::interned]
pub struct Project {
    pub root: Document,

    #[return_ref]
    pub dependencies: Vec<Dependency>,
}

impl Project {
    pub fn documents<'db>(self, db: &'db dyn Db) -> impl Iterator<Item = Document> + 'db {
        self.dependencies(db)
            .iter()
            .map(|dependency| dependency.document(db))
    }
}

#[salsa::tracked]
pub fn project_of(db: &dyn Db, root: Document) -> Project {
    let mut results = Vec::new();
    let mut visited = FxHashSet::default();
    let mut stack = vec![Dependency::new(db, root, root.file(db))];
    while let Some(dependency) = stack.pop() {
        if !visited.insert(dependency.document(db)) {
            break;
        }

        results.push(dependency);
        stack.extend(explicit_dependencies_of(db, dependency));
        stack.extend(implicit_dependencies_of(db, dependency));
    }

    Project::new(db, root, results)
}

#[salsa::tracked]
pub struct ProjectGroup {
    #[return_ref]
    pub projects: Vec<Project>,
}

#[salsa::tracked]
impl ProjectGroup {
    #[salsa::tracked(return_ref)]
    pub fn union(self, db: &dyn Db) -> Vec<Document> {
        self.projects(db)
            .iter()
            .flat_map(|project| project.documents(db))
            .unique()
            .collect()
    }
}

#[salsa::tracked]
pub fn project_group_of(db: &dyn Db, child: Document) -> ProjectGroup {
    let ancestors = child
        .file(db)
        .path(db)
        .into_iter()
        .flat_map(|path| path.ancestors());

    fn find(db: &dyn Db, child: Document) -> Vec<Project> {
        db.workspace()
            .iter()
            .filter(|doc| doc.can_be_root(db))
            .map(|root| project_of(db, root))
            .filter(|project| project.documents(db).contains(&child))
            .collect()
    }

    for path in ancestors {
        let projects = find(db, child);

        if !projects.is_empty() {
            return ProjectGroup::new(db, projects);
        }

        let files = std::fs::read_dir(path)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().map_or(false, |ty| ty.is_file()))
            .filter_map(|entry| Url::from_file_path(entry.path()).ok())
            .map(|uri| FileId::new(db, uri))
            .filter(|file| file.language(db) == Some(Language::Tex));

        for file in files {
            db.workspace().load(db, file);
        }
    }

    let mut projects = find(db, child);
    if projects.is_empty() {
        projects = vec![project_of(db, child)];
    }

    ProjectGroup::new(db, projects)
}
