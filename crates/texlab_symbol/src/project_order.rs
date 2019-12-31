use petgraph::algo::tarjan_scc;
use petgraph::{Directed, Graph};
use std::collections::HashSet;
use std::sync::Arc;
use texlab_protocol::Uri;
use texlab_syntax::*;
use texlab_workspace::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProjectOrdering {
    ordering: Vec<Arc<Document>>,
}

impl ProjectOrdering {
    pub fn new(workspace: &Workspace) -> Self {
        let mut ordering = Vec::new();
        let connected_components = Self::connected_components(workspace);
        for connected_component in connected_components {
            let graph = Self::build_dependency_graph(&connected_component);

            let mut visited = HashSet::new();
            let root_index = *graph.node_weight(tarjan_scc(&graph)[0][0]).unwrap();
            let mut stack = vec![Arc::clone(&connected_component[root_index])];

            while let Some(document) = stack.pop() {
                if !visited.insert(document.uri.as_str().to_owned()) {
                    continue;
                }

                ordering.push(Arc::clone(&document));
                if let SyntaxTree::Latex(tree) = &document.tree {
                    for include in tree.includes.iter().rev() {
                        for targets in &include.all_targets {
                            for target in targets {
                                if let Some(child) = workspace.find(target) {
                                    stack.push(child);
                                }
                            }
                        }
                    }
                }
            }
        }

        Self { ordering }
    }

    fn connected_components(workspace: &Workspace) -> Vec<Vec<Arc<Document>>> {
        let mut components = Vec::new();
        let mut visited = HashSet::new();
        for root in &workspace.documents {
            if !visited.insert(root.uri.clone()) {
                continue;
            }

            let component = workspace.related_documents(&root.uri);
            for document in &component {
                visited.insert(document.uri.clone());
            }
            components.push(component);
        }
        components
    }

    fn build_dependency_graph(documents: &[Arc<Document>]) -> Graph<usize, (), Directed> {
        let mut graph = Graph::new();
        let nodes: Vec<_> = (0..documents.len()).map(|i| graph.add_node(i)).collect();

        for (i, document) in documents.iter().enumerate() {
            if let SyntaxTree::Latex(tree) = &document.tree {
                for targets in tree
                    .includes
                    .iter()
                    .flat_map(|include| &include.all_targets)
                {
                    for target in targets {
                        if let Some(j) = documents.iter().position(|doc| doc.uri == *target) {
                            graph.add_edge(nodes[j], nodes[i], ());
                            break;
                        }
                    }
                }
            }
        }
        graph
    }

    pub fn get(&self, uri: &Uri) -> usize {
        self.ordering
            .iter()
            .position(|doc| doc.uri == *uri)
            .unwrap_or(std::usize::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cycles() {
        let mut builder = TestWorkspaceBuilder::new();
        let a = builder.add_document("a.tex", "");
        let b = builder.add_document("b.tex", "");
        let c = builder.add_document("c.tex", "\\include{b}\\include{a}");

        let project_ordering = ProjectOrdering::new(&builder.workspace);

        assert_eq!(project_ordering.get(&a), 2);
        assert_eq!(project_ordering.get(&b), 1);
        assert_eq!(project_ordering.get(&c), 0);
    }

    #[test]
    fn test_cycles() {
        let mut builder = TestWorkspaceBuilder::new();
        let a = builder.add_document("a.tex", "\\include{b}");
        let b = builder.add_document("b.tex", "\\include{a}");
        let c = builder.add_document("c.tex", "\\include{a}");

        let project_ordering = ProjectOrdering::new(&builder.workspace);

        assert_eq!(project_ordering.get(&a), 1);
        assert_eq!(project_ordering.get(&b), 2);
        assert_eq!(project_ordering.get(&c), 0);
    }

    #[test]
    fn test_multiple_roots() {
        let mut builder = TestWorkspaceBuilder::new();
        let a = builder.add_document("a.tex", "\\include{b}");
        let b = builder.add_document("b.tex", "");
        let c = builder.add_document("c.tex", "");
        let d = builder.add_document("d.tex", "\\include{c}");

        let project_ordering = ProjectOrdering::new(&builder.workspace);

        assert_eq!(project_ordering.get(&a), 0);
        assert_eq!(project_ordering.get(&b), 1);
        assert_eq!(project_ordering.get(&d), 2);
        assert_eq!(project_ordering.get(&c), 3);
    }
}
