use itertools::Itertools;
use rustc_hash::FxHashSet;

use crate::{Document, Workspace};

use super::graph;

#[derive(Debug, Clone)]
pub struct Project<'a> {
    pub documents: FxHashSet<&'a Document>,
}

impl<'a> Project<'a> {
    pub fn from_child(workspace: &'a Workspace, child: &'a Document) -> Self {
        let mut documents = FxHashSet::default();
        for start in workspace.iter() {
            let graph = graph::Graph::new(workspace, start);
            if graph.preorder().contains(&child) {
                documents.extend(graph.preorder());
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
            let graph = graph::Graph::new(workspace, parent);
            let mut nodes = graph.preorder();
            nodes.contains(&child)
        })
        .collect()
}
