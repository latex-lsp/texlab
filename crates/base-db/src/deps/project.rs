use itertools::Itertools;
use rustc_hash::FxHashSet;

use crate::{Document, Workspace};

#[derive(Debug, Clone)]
pub struct Project<'a> {
    pub documents: FxHashSet<&'a Document>,
}

impl<'a> Project<'a> {
    pub fn from_child(workspace: &'a Workspace, child: &'a Document) -> Self {
        let mut documents = FxHashSet::default();
        for graph in workspace.graphs().values() {
            if graph.preorder(workspace).contains(&child) {
                documents.extend(graph.preorder(workspace));
            }
        }

        Self { documents }
    }
}

pub fn parents<'a>(workspace: &'a Workspace, child: &'a Document) -> FxHashSet<&'a Document> {
    workspace
        .iter()
        .filter(|document| {
            document
                .data
                .as_tex()
                .map_or(false, |data| data.semantics.can_be_root)
        })
        .filter(|parent| {
            let graph = &workspace.graphs()[&parent.uri];
            let mut nodes = graph.preorder(workspace);
            nodes.contains(&child)
        })
        .collect()
}
