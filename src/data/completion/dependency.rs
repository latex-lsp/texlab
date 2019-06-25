use crate::data::completion::component::LatexComponent;
use crate::tex;
use crate::tex::build_test_code_header;
use crate::tex::resolver::TexResolver;
use futures::lock::*;
use once_cell::sync::Lazy;
use petgraph::algo::tarjan_scc;
use petgraph::Graph;
use regex::Regex;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

static FILE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"[^\s\r\n]+\.(sty|tex|def|cls)"#).unwrap());

#[derive(Debug, Eq, Clone)]
pub struct LatexDependency {
    pub file: PathBuf,
    pub includes: Vec<PathBuf>,
}

impl Hash for LatexDependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file.hash(state)
    }
}

impl PartialEq for LatexDependency {
    fn eq(&self, other: &Self) -> bool {
        return self.file == other.file;
    }
}

impl LatexDependency {
    pub async fn load<'a>(file: &'a Path, resolver: &'a TexResolver) -> Self {
        let includes = match Self::find_includes(file, resolver).await {
            Some(includes) => includes,
            None => vec![file.to_owned()],
        };

        LatexDependency {
            file: file.to_owned(),
            includes,
        }
    }

    pub fn references(&self) -> impl Iterator<Item = &Path> {
        let file = self.file.clone();
        self.includes
            .iter()
            .filter(move |include| **include != file && Self::is_dependency_file(include))
            .map(|include| include.as_path())
    }

    pub async fn into_components<'a>(
        self,
        resolver: &'a TexResolver,
        components_by_name: &'a Mutex<HashMap<String, Arc<LatexComponent>>>,
    ) -> Vec<Vec<Arc<LatexDependency>>> {
        let mut missing_refs = Vec::new();
        for file in self.references() {
            if {
                !components_by_name
                    .lock()
                    .await
                    .contains_key(file.file_name().unwrap().to_str().unwrap())
            } {
                let reference = Self::load(file, resolver).await;
                missing_refs.push(reference);
            }
        }

        let mut graph = Graph::new();
        graph.add_node(Arc::new(self));
        for reference in missing_refs {
            graph.add_node(Arc::new(reference));
        }

        let edges: Vec<_> = graph
            .node_indices()
            .map(|i| (i, &graph[i]))
            .flat_map(|(i, dep)| {
                dep.references()
                    .filter_map(|ref_| graph.node_indices().find(|j| &graph[*j].file == ref_))
                    .map(move |j| (i, j))
            })
            .collect();

        for (i, j) in edges {
            graph.add_edge(i, j, ());
        }

        tarjan_scc(&graph)
            .into_iter()
            .map(|comp| comp.into_iter().map(|i| Arc::clone(&graph[i])).collect())
            .collect()
    }

    fn is_dependency_file(file: &Path) -> bool {
        match file.extension().unwrap().to_str().unwrap() {
            "sty" | "cls" => true,
            _ => false,
        }
    }

    async fn find_includes<'a>(file: &'a Path, resolver: &'a TexResolver) -> Option<Vec<PathBuf>> {
        let mut code = build_test_code_header(file)?;
        code += "\\listfiles\n";
        code += "\\begin{document} \\end{document}";
        let result = tex::compile("code.tex", &code, file.into()).await.ok()?;

        let start_index = result.log.find("*File List*")?;
        let extension = file.extension().unwrap();
        let includes: Vec<_> = FILE_REGEX
            .find_iter(&result.log[start_index..])
            .map(|x| x.as_str())
            .filter(|x| extension == "cls" || *x != "article.cls")
            .filter_map(|x| resolver.files_by_name.get(&x.to_owned()).cloned())
            .collect();

        Some(includes)
    }
}
