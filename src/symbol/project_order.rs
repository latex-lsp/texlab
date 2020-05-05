use crate::{
    protocol::{Options, Uri},
    workspace::{Document, DocumentContent, Snapshot},
};
use petgraph::{algo::tarjan_scc, Directed, Graph};
use std::{collections::HashSet, path::Path, sync::Arc, usize};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProjectOrdering {
    ordering: Vec<Arc<Document>>,
}

impl ProjectOrdering {
    pub fn get(&self, uri: &Uri) -> usize {
        self.ordering
            .iter()
            .position(|doc| doc.uri == *uri)
            .unwrap_or(usize::MAX)
    }

    pub fn analyze(snapshot: &Snapshot, options: &Options, current_dir: &Path) -> Self {
        let mut ordering = Vec::new();
        let comps = Self::connected_components(snapshot, options, current_dir);
        for comp in comps {
            let graph = Self::build_dependency_graph(&comp);

            let mut visited = HashSet::new();
            let root_index = *graph.node_weight(tarjan_scc(&graph)[0][0]).unwrap();
            let mut stack = vec![Arc::clone(&comp[root_index])];

            while let Some(doc) = stack.pop() {
                if !visited.insert(doc.uri.as_str().to_owned()) {
                    continue;
                }

                ordering.push(Arc::clone(&doc));
                if let DocumentContent::Latex(tree) = &doc.content {
                    for include in tree.includes.iter().rev() {
                        for targets in &include.all_targets {
                            for target in targets {
                                if let Some(child) = snapshot.find(target) {
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

    fn connected_components(
        snapshot: &Snapshot,
        options: &Options,
        current_dir: &Path,
    ) -> Vec<Vec<Arc<Document>>> {
        let mut comps = Vec::new();
        let mut visited = HashSet::new();
        for root in &snapshot.0 {
            if !visited.insert(root.uri.clone()) {
                continue;
            }

            let comp = snapshot.relations(&root.uri, options, current_dir);
            for document in &comp {
                visited.insert(document.uri.clone());
            }
            comps.push(comp);
        }
        comps
    }

    fn build_dependency_graph(docs: &[Arc<Document>]) -> Graph<usize, (), Directed> {
        let mut graph = Graph::new();
        let nodes: Vec<_> = (0..docs.len()).map(|i| graph.add_node(i)).collect();

        for (i, doc) in docs.iter().enumerate() {
            if let DocumentContent::Latex(tree) = &doc.content {
                for targets in tree
                    .includes
                    .iter()
                    .flat_map(|include| &include.all_targets)
                {
                    for target in targets {
                        if let Some(j) = docs.iter().position(|doc| doc.uri == *target) {
                            graph.add_edge(nodes[j], nodes[i], ());
                            break;
                        }
                    }
                }
            }
        }
        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        tex::{Language, Resolver},
        workspace::DocumentParams,
    };
    use std::env;

    fn create_simple_document(uri: &Uri, language: Language, text: &str) -> Arc<Document> {
        Arc::new(Document::open(DocumentParams {
            uri: uri.clone(),
            text: text.into(),
            language,
            resolver: &Resolver::default(),
            options: &Options::default(),
            current_dir: &env::current_dir().unwrap(),
        }))
    }

    #[test]
    fn no_cycles() {
        let a = Uri::parse("http://example.com/a.tex").unwrap();
        let b = Uri::parse("http://example.com/b.tex").unwrap();
        let c = Uri::parse("http://example.com/c.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            create_simple_document(&a, Language::Latex, ""),
            create_simple_document(&b, Language::Latex, ""),
            create_simple_document(&c, Language::Latex, r#"\include{b}\include{a}"#),
        ];

        let current_dir = env::current_dir().unwrap();
        let ordering = ProjectOrdering::analyze(&snapshot, &Options::default(), &current_dir);

        assert_eq!(ordering.get(&a), 2);
        assert_eq!(ordering.get(&b), 1);
        assert_eq!(ordering.get(&c), 0);
    }

    #[test]
    fn cycles() {
        let a = Uri::parse("http://example.com/a.tex").unwrap();
        let b = Uri::parse("http://example.com/b.tex").unwrap();
        let c = Uri::parse("http://example.com/c.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            create_simple_document(&a, Language::Latex, r#"\include{b}"#),
            create_simple_document(&b, Language::Latex, r#"\include{a}"#),
            create_simple_document(&c, Language::Latex, r#"\include{a}"#),
        ];

        let current_dir = env::current_dir().unwrap();
        let ordering = ProjectOrdering::analyze(&snapshot, &Options::default(), &current_dir);

        assert_eq!(ordering.get(&a), 1);
        assert_eq!(ordering.get(&b), 2);
        assert_eq!(ordering.get(&c), 0);
    }

    #[test]
    fn multiple_roots() {
        let a = Uri::parse("http://example.com/a.tex").unwrap();
        let b = Uri::parse("http://example.com/b.tex").unwrap();
        let c = Uri::parse("http://example.com/c.tex").unwrap();
        let d = Uri::parse("http://example.com/d.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            create_simple_document(&a, Language::Latex, r#"\include{b}"#),
            create_simple_document(&b, Language::Latex, ""),
            create_simple_document(&c, Language::Latex, ""),
            create_simple_document(&d, Language::Latex, r#"\include{c}"#),
        ];

        let current_dir = env::current_dir().unwrap();
        let ordering = ProjectOrdering::analyze(&snapshot, &Options::default(), &current_dir);

        assert_eq!(ordering.get(&a), 0);
        assert_eq!(ordering.get(&b), 1);
        assert_eq!(ordering.get(&d), 2);
        assert_eq!(ordering.get(&c), 3);
    }
}
