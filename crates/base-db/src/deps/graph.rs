use std::{ffi::OsStr, path::PathBuf, rc::Rc};

use distro::Language;
use itertools::Itertools;
use once_cell::sync::Lazy;
use percent_encoding::percent_decode_str;
use rustc_hash::FxHashSet;
use url::Url;

use crate::{semantics, util, Document, Workspace};

use super::ProjectRoot;

pub static HOME_DIR: Lazy<Option<PathBuf>> = Lazy::new(dirs::home_dir);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Edge {
    pub source: Url,
    pub target: Url,
    pub data: EdgeData,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum EdgeData {
    DirectLink(Box<DirectLinkData>),
    AdditionalFiles,
    Artifact,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DirectLinkData {
    pub link: semantics::tex::Link,
    pub new_root: Option<ProjectRoot>,
}

#[derive(Debug, Clone, Copy)]
struct Start<'a, 'b> {
    source: &'a Document,
    root: &'b ProjectRoot,
}

#[derive(Debug)]
pub struct Graph {
    pub missing: Vec<Url>,
    pub edges: Vec<Edge>,
    pub start: Url,
}

impl Graph {
    pub fn new(workspace: &Workspace, start: &Document) -> Self {
        let mut graph = Self {
            missing: Vec::new(),
            edges: Vec::new(),
            start: start.uri.clone(),
        };

        let root = ProjectRoot::walk_and_find(workspace, &start.dir);

        let mut stack = vec![(start, Rc::new(root))];
        let mut visited = FxHashSet::default();

        while let Some((source, root)) = stack.pop() {
            let index = graph.edges.len();

            graph.process(
                workspace,
                Start {
                    source,
                    root: &root,
                },
            );

            for edge in &graph.edges[index..] {
                if visited.insert(edge.target.clone()) {
                    let new_root = match &edge.data {
                        EdgeData::DirectLink(data) => data.new_root.clone(),
                        _ => None,
                    };

                    let new_root = new_root.map_or_else(|| Rc::clone(&root), Rc::new);

                    stack.push((workspace.lookup(&edge.target).unwrap(), new_root));
                }
            }
        }

        graph
    }

    pub fn preorder<'a: 'b, 'b>(
        &'b self,
        workspace: &'a Workspace,
    ) -> impl DoubleEndedIterator<Item = &'a Document> + '_ {
        std::iter::once(&self.start)
            .chain(self.edges.iter().map(|group| &group.target))
            .unique()
            .filter_map(|uri| workspace.lookup(uri))
    }

    fn process(&mut self, workspace: &Workspace, start: Start) {
        self.add_direct_links(workspace, start);
        self.add_artifacts(workspace, start);
        self.add_additional_files(workspace, start);
    }

    fn add_additional_files(&mut self, workspace: &Workspace, start: Start) {
        for uri in &start.root.additional_files {
            match workspace.lookup(uri) {
                Some(target) => {
                    self.edges.push(Edge {
                        source: start.source.uri.clone(),
                        target: target.uri.clone(),
                        data: EdgeData::AdditionalFiles,
                    });
                }
                None => {
                    self.missing.push(uri.clone());
                }
            }
        }
    }

    fn add_direct_links(&mut self, workspace: &Workspace, start: Start) -> Option<()> {
        let data = start.source.data.as_tex()?;

        for link in &data.semantics.links {
            self.add_direct_link(workspace, start, link);
        }

        Some(())
    }

    fn add_direct_link(
        &mut self,
        workspace: &Workspace,
        start: Start,
        link: &semantics::tex::Link,
    ) {
        let home_dir = HOME_DIR.as_deref();

        let stem = &link.path.text;
        let mut file_names = vec![stem.clone()];
        link.kind
            .extensions()
            .iter()
            .map(|ext| format!("{stem}.{ext}"))
            .for_each(|name| file_names.push(name));

        let file_name_db = &workspace.distro().file_name_db;
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
            .flat_map(|file_name| {
                util::expand_relative_path(file_name, &start.root.src_dir, workspace.folders())
            })
            .chain(distro_files)
        {
            match workspace.lookup(&target_uri) {
                Some(target) => {
                    let new_root = link
                        .base_dir
                        .as_deref()
                        .and_then(|path| start.root.src_dir.join(path).ok())
                        .map(|dir| ProjectRoot::from_config(workspace, &dir));

                    let link_data = DirectLinkData {
                        link: link.clone(),
                        new_root,
                    };

                    self.edges.push(Edge {
                        source: start.source.uri.clone(),
                        target: target.uri.clone(),
                        data: EdgeData::DirectLink(Box::new(link_data)),
                    });

                    break;
                }
                None => {
                    self.missing.push(target_uri);
                }
            };
        }
    }

    fn add_artifacts(&mut self, workspace: &Workspace, start: Start) {
        if start.source.language != Language::Tex {
            return;
        }

        let root = start.root;
        let relative_path = root.compile_dir.make_relative(&start.source.uri).unwrap();

        self.add_artifact(
            workspace,
            start.source,
            &root.aux_dir.join(&relative_path).unwrap(),
            "aux",
        );

        self.add_artifact(workspace, start.source, &root.aux_dir, "aux");
        self.add_artifact(workspace, start.source, &root.compile_dir, "aux");

        self.add_artifact(
            workspace,
            start.source,
            &root.log_dir.join(&relative_path).unwrap(),
            "log",
        );

        self.add_artifact(workspace, start.source, &root.log_dir, "log");
        self.add_artifact(workspace, start.source, &root.compile_dir, "log");
    }

    fn add_artifact(
        &mut self,
        workspace: &Workspace,
        source: &Document,
        dir: &Url,
        extension: &str,
    ) {
        let mut path = PathBuf::from(
            percent_decode_str(source.uri.path())
                .decode_utf8_lossy()
                .as_ref(),
        );

        path.set_extension(extension);
        let Some(target_uri) = path
            .file_name()
            .and_then(OsStr::to_str)
            .and_then(|name| dir.join(name).ok())
        else {
            return;
        };

        match workspace.lookup(&target_uri) {
            Some(target) => {
                self.edges.push(Edge {
                    source: source.uri.clone(),
                    target: target.uri.clone(),
                    data: EdgeData::Artifact,
                });
            }
            None => {
                self.missing.push(target_uri);
            }
        }
    }
}
