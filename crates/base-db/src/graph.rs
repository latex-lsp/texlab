use std::{ffi::OsStr, path::PathBuf};

use distro::Language;
use itertools::Itertools;
use once_cell::sync::Lazy;
use percent_encoding::percent_decode_str;
use rustc_hash::FxHashSet;
use url::Url;

use crate::{semantics, Document, DocumentData, Workspace};

pub static HOME_DIR: Lazy<Option<PathBuf>> = Lazy::new(dirs::home_dir);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Edge<'a> {
    pub source: &'a Document,
    pub target: &'a Document,
    pub weight: Option<EdgeWeight<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct EdgeWeight<'a> {
    pub link: &'a semantics::tex::Link,
    pub old_base_dir: Url,
    pub new_base_dir: Url,
}

#[derive(Debug)]
pub struct Graph<'a> {
    pub workspace: &'a Workspace,
    pub start: &'a Document,
    pub edges: Vec<Edge<'a>>,
    pub missing: Vec<Url>,
}

impl<'a> Graph<'a> {
    pub fn new(workspace: &'a Workspace, start: &'a Document) -> Self {
        let mut graph = Self {
            workspace,
            start,
            edges: Vec::new(),
            missing: Vec::new(),
        };

        let base_dir = workspace.current_dir(&start.dir);
        let mut stack = vec![(start, base_dir)];
        let mut visited = FxHashSet::default();

        while let Some((source, base_dir)) = stack.pop() {
            let index = graph.edges.len();
            graph.add_explicit_edges(source, &base_dir);
            for edge in &graph.edges[index..] {
                let Some(weight) = edge.weight.as_ref() else {
                    continue;
                };

                if visited.insert(&edge.target.uri) {
                    stack.push((edge.target, weight.new_base_dir.clone()));
                }
            }

            graph.add_implicit_edges(source, &base_dir);
        }

        graph
    }

    pub fn preorder(&self) -> impl DoubleEndedIterator<Item = &'a Document> + '_ {
        std::iter::once(self.start)
            .chain(self.edges.iter().map(|group| group.target))
            .unique_by(|document| &document.uri)
    }

    fn add_explicit_edges(&mut self, source: &'a Document, base_dir: &Url) {
        let DocumentData::Tex(data) = &source.data else {
            return;
        };

        for link in &data.semantics.links {
            self.add_link(source, base_dir, link);
        }
    }

    fn add_link(&mut self, source: &'a Document, base_dir: &Url, link: &'a semantics::tex::Link) {
        let home_dir = HOME_DIR.as_deref();

        let stem = &link.path.text;
        let mut file_names = vec![stem.clone()];
        link.kind
            .extensions()
            .iter()
            .map(|ext| format!("{stem}.{ext}"))
            .for_each(|name| file_names.push(name));

        let file_name_db = &self.workspace.distro().file_name_db;
        let distro_files = file_names
            .iter()
            .filter_map(|name| file_name_db.get(name))
            .filter(|path| {
                home_dir.map_or(false, |dir| path.starts_with(dir))
                    || Language::from_path(path) == Some(Language::Bib)
            })
            .flat_map(Url::from_file_path);

        for target_uri in file_names
            .iter()
            .flat_map(|file_name| base_dir.join(file_name))
            .chain(distro_files)
        {
            match self.workspace.lookup(&target_uri) {
                Some(target) => {
                    let new_base_dir = link
                        .base_dir
                        .as_deref()
                        .and_then(|path| base_dir.join(path).ok())
                        .unwrap_or_else(|| base_dir.clone());

                    let weight = Some(EdgeWeight {
                        link,
                        old_base_dir: base_dir.clone(),
                        new_base_dir,
                    });

                    self.edges.push(Edge {
                        source,
                        target,
                        weight,
                    });
                }
                None => {
                    self.missing.push(target_uri);
                }
            };
        }
    }

    fn add_implicit_edges(&mut self, source: &'a Document, base_dir: &Url) {
        if source.language == Language::Tex {
            let config = &self.workspace.config().build;
            let aux_dir = self.workspace.output_dir(base_dir, config.aux_dir.clone());
            let log_dir = self.workspace.output_dir(base_dir, config.log_dir.clone());

            let relative_path = base_dir.make_relative(&source.uri).unwrap();

            self.add_artifact(source, &aux_dir.join(&relative_path).unwrap(), "aux");
            self.add_artifact(source, &aux_dir, "aux");
            self.add_artifact(source, base_dir, "aux");

            self.add_artifact(source, &log_dir.join(&relative_path).unwrap(), "log");
            self.add_artifact(source, &log_dir, "log");
            self.add_artifact(source, base_dir, "log");
        }
    }

    fn add_artifact(&mut self, source: &'a Document, base_dir: &Url, extension: &str) {
        let mut path = PathBuf::from(
            percent_decode_str(source.uri.path())
                .decode_utf8_lossy()
                .as_ref(),
        );

        path.set_extension(extension);
        let Some(target_uri) = path
            .file_name()
            .and_then(OsStr::to_str)
            .and_then(|name| base_dir.join(name).ok())
        else {
            return;
        };

        match self.workspace.lookup(&target_uri) {
            Some(target) => {
                self.edges.push(Edge {
                    source,
                    target,
                    weight: None,
                });
            }
            None => {
                self.missing.push(target_uri);
            }
        }
    }
}
