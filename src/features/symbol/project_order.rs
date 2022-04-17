use std::{sync::Arc, usize};

use petgraph::{algo::tarjan_scc, Directed, Graph};
use rustc_hash::FxHashSet;

use crate::{Uri, Workspace, WorkspaceSubset};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProjectOrdering {
    ordering: Vec<Arc<Uri>>,
}

impl ProjectOrdering {
    pub fn get(&self, uri: &Uri) -> usize {
        self.ordering
            .iter()
            .position(|u| u.as_ref() == uri)
            .unwrap_or(usize::MAX)
    }
}

impl From<&dyn Workspace> for ProjectOrdering {
    fn from(workspace: &dyn Workspace) -> Self {
        let mut ordering = Vec::new();
        let uris: FxHashSet<Arc<Uri>> = workspace
            .documents()
            .into_iter()
            .map(|document| Arc::clone(&document.uri))
            .collect();

        let comps = connected_components(workspace);
        for comp in comps {
            let graph = build_dependency_graph(&comp);

            let mut visited = FxHashSet::default();
            let root_index = *graph.node_weight(tarjan_scc(&graph)[0][0]).unwrap();
            let mut stack = vec![Arc::clone(&comp.documents[root_index].uri)];

            while let Some(uri) = stack.pop() {
                if !visited.insert(Arc::clone(&uri)) {
                    continue;
                }

                ordering.push(Arc::clone(&uri));
                if let Some(document) = workspace.get(&uri) {
                    if let Some(data) = document.data.as_latex() {
                        for link in data.extras.explicit_links.iter().rev() {
                            for target in &link.targets {
                                if uris.contains(target.as_ref()) {
                                    stack.push(Arc::clone(target));
                                }
                            }
                        }
                    }
                }
            }
        }

        Self { ordering }
    }
}

fn connected_components(workspace: &dyn Workspace) -> Vec<WorkspaceSubset> {
    let mut components = Vec::new();
    let mut visited = FxHashSet::default();
    for root_document in workspace.documents() {
        if !visited.insert(Arc::clone(&root_document.uri)) {
            continue;
        }

        let subset = workspace.subset(Arc::clone(&root_document.uri)).unwrap();
        for document in &subset.documents {
            visited.insert(Arc::clone(&document.uri));
        }
        components.push(subset);
    }
    components
}

fn build_dependency_graph(subset: &WorkspaceSubset) -> Graph<usize, (), Directed> {
    let mut graph = Graph::new();
    let nodes: Vec<_> = (0..subset.documents.len())
        .map(|i| graph.add_node(i))
        .collect();

    for (i, document) in subset.documents.iter().enumerate() {
        if let Some(data) = document.data.as_latex() {
            for link in &data.extras.explicit_links {
                for target in &link.targets {
                    if let Some(j) = subset
                        .documents
                        .iter()
                        .position(|document| document.uri.as_ref() == target.as_ref())
                    {
                        graph.add_edge(nodes[j], nodes[i], ());
                        break;
                    }
                }
            }
        }
    }
    graph
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;

    use crate::{create_workspace_fast, DocumentLanguage, ServerContext, WorkspaceSource};

    use super::*;

    #[test]
    fn test_no_cycles() -> Result<()> {
        let workspace: Arc<dyn Workspace> = Arc::new(create_workspace_fast(Arc::new(
            ServerContext::new(std::env::temp_dir()),
        ))?);

        let a = workspace.open(
            Arc::new(Uri::parse("http://example.com/a.tex")?),
            Arc::new(String::new()),
            DocumentLanguage::Latex,
            WorkspaceSource::Client,
        );

        let b = workspace.open(
            Arc::new(Uri::parse("http://example.com/b.tex")?),
            Arc::new(String::new()),
            DocumentLanguage::Latex,
            WorkspaceSource::Client,
        );

        let c = workspace.open(
            Arc::new(Uri::parse("http://example.com/c.tex")?),
            Arc::new(r#"\include{b}\include{a}"#.to_string()),
            DocumentLanguage::Latex,
            WorkspaceSource::Client,
        );

        let ordering = ProjectOrdering::from(workspace.as_ref());

        assert_eq!(ordering.get(&a.uri), 2);
        assert_eq!(ordering.get(&b.uri), 1);
        assert_eq!(ordering.get(&c.uri), 0);
        Ok(())
    }

    #[test]
    fn test_cycles() -> Result<()> {
        let workspace: Arc<dyn Workspace> = Arc::new(create_workspace_fast(Arc::new(
            ServerContext::new(std::env::temp_dir()),
        ))?);

        let a = workspace.open(
            Arc::new(Uri::parse("http://example.com/a.tex")?),
            Arc::new(r#"\include{b}"#.to_string()),
            DocumentLanguage::Latex,
            WorkspaceSource::Client,
        );

        let b = workspace.open(
            Arc::new(Uri::parse("http://example.com/b.tex")?),
            Arc::new(r#"\include{a}"#.to_string()),
            DocumentLanguage::Latex,
            WorkspaceSource::Client,
        );

        let c = workspace.open(
            Arc::new(Uri::parse("http://example.com/c.tex")?),
            Arc::new(r#"\include{a}"#.to_string()),
            DocumentLanguage::Latex,
            WorkspaceSource::Client,
        );

        let ordering = ProjectOrdering::from(workspace.as_ref());

        assert_eq!(ordering.get(&a.uri), 1);
        assert_eq!(ordering.get(&b.uri), 2);
        assert_eq!(ordering.get(&c.uri), 0);
        Ok(())
    }

    #[test]
    fn test_multiple_roots() -> Result<()> {
        let workspace: Arc<dyn Workspace> = Arc::new(create_workspace_fast(Arc::new(
            ServerContext::new(std::env::temp_dir()),
        ))?);

        let a = workspace.open(
            Arc::new(Uri::parse("http://example.com/a.tex")?),
            Arc::new(r#"\include{b}"#.to_string()),
            DocumentLanguage::Latex,
            WorkspaceSource::Client,
        );

        let b = workspace.open(
            Arc::new(Uri::parse("http://example.com/b.tex")?),
            Arc::new(r#""#.to_string()),
            DocumentLanguage::Latex,
            WorkspaceSource::Client,
        );

        let c = workspace.open(
            Arc::new(Uri::parse("http://example.com/c.tex")?),
            Arc::new(r#""#.to_string()),
            DocumentLanguage::Latex,
            WorkspaceSource::Client,
        );

        let d = workspace.open(
            Arc::new(Uri::parse("http://example.com/d.tex")?),
            Arc::new(r#"\include{c}"#.to_string()),
            DocumentLanguage::Latex,
            WorkspaceSource::Client,
        );

        let ordering = ProjectOrdering::from(workspace.as_ref());

        assert!(ordering.get(&a.uri) < ordering.get(&b.uri));
        assert!(ordering.get(&d.uri) < ordering.get(&c.uri));
        Ok(())
    }
}
