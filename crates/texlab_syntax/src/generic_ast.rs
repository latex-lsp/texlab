use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct AstNodeIndex(usize);

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Ast<T> {
    nodes: Vec<T>,
    edges: Vec<Vec<AstNodeIndex>>,
}

impl<T> Ast<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn nodes(&self) -> Vec<AstNodeIndex> {
        let mut nodes = Vec::new();
        for i in 0..self.nodes.len() {
            nodes.push(AstNodeIndex(i));
        }
        nodes
    }

    pub fn add_node(&mut self, value: T) -> AstNodeIndex {
        let node = AstNodeIndex(self.nodes.len());
        self.nodes.push(value);
        self.edges.push(Vec::new());
        node
    }

    pub fn add_edge(&mut self, parent: AstNodeIndex, child: AstNodeIndex) {
        self.edges[parent.0].push(child);
    }

    pub fn children<'a>(&'a self, parent: AstNodeIndex) -> impl Iterator<Item = AstNodeIndex> + 'a {
        self.edges[parent.0].iter().map(|child| *child)
    }
}

impl<T> Index<AstNodeIndex> for Ast<T> {
    type Output = T;

    fn index(&self, index: AstNodeIndex) -> &Self::Output {
        &self.nodes[index.0]
    }
}
