use crate::syntax::*;
use crate::workspace::*;
use petgraph::algo::tarjan_scc;
use petgraph::Graph;

pub struct ProjectOrder(Vec<usize>);

impl ProjectOrder {
    pub fn new(workspace: &Workspace) -> Self {
        let mut graph = Graph::new();
        let nodes: Vec<_> = (0..workspace.documents.len())
            .map(|i| graph.add_node(i))
            .collect();

        for (i, document) in workspace.documents.iter().enumerate() {
            if let SyntaxTree::Latex(tree) = &document.tree {
                for targets in tree
                    .includes
                    .iter()
                    .flat_map(|include| &include.all_targets)
                {
                    for target in targets {
                        if let Some(j) = workspace
                            .documents
                            .iter()
                            .position(|doc| doc.uri == *target)
                        {
                            graph.add_edge(nodes[j], nodes[i], ());
                            break;
                        }
                    }
                }
            }
        }

        let order = tarjan_scc(&graph)
            .into_iter()
            .flat_map(|component| component)
            .map(|node| *graph.node_weight(node).unwrap())
            .collect();

        Self(order)
    }

    pub fn rank(&self, workspace: &Workspace, uri: &Uri) -> Option<usize> {
        let i = workspace.documents.iter().position(|doc| doc.uri == *uri)?;
        self.0.iter().position(|j| i == *j)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preserves_order() {
        let mut builder = WorkspaceBuilder::new();
        builder.document("main.tex", "\\include{foo}\n\\include{bar}\n\\include{baz}");
        builder.document("foo.tex", "");
        builder.document("bar.tex", "");
        builder.document("baz.tex", "");
        assert_eq!(ProjectOrder::new(&builder.workspace).0, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_cycle() {
        let mut builder = WorkspaceBuilder::new();
        builder.document("foo.tex", "\\include{bar}");
        builder.document("bar.tex", "\\include{baz}");
        builder.document("baz.tex", "\\include{bar}");
        assert_eq!(ProjectOrder::new(&builder.workspace).0, vec![0, 2, 1]);
    }

    #[test]
    fn test_nesting() {
        let mut builder = WorkspaceBuilder::new();
        builder.document("foo.tex", "\\include{bar}");
        builder.document("bar.tex", "\\include{baz}");
        builder.document("baz.tex", "");
        assert_eq!(ProjectOrder::new(&builder.workspace).0, vec![0, 1, 2]);
    }
}
