use super::components::COMPONENT_DATABASE;
use super::document::Document;
use path_clean::PathClean;
use petgraph::visit::Dfs;
use petgraph::Graph;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use texlab_distro::{Language, Resolver};
use texlab_protocol::*;
use texlab_syntax::*;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Workspace {
    pub documents: Vec<Arc<Document>>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    pub fn find(&self, uri: &Uri) -> Option<Arc<Document>> {
        self.documents
            .iter()
            .find(|document| &document.uri == uri)
            .map(|document| Arc::clone(&document))
    }

    pub fn related_documents(&self, uri: &Uri, options: &Options) -> Vec<Arc<Document>> {
        let mut graph = Graph::new_undirected();
        let mut indices_by_uri = HashMap::new();
        for document in &self.documents {
            indices_by_uri.insert(&document.uri, graph.add_node(document));
        }

        for parent in self.documents.iter().filter(|doc| doc.is_file()) {
            if let SyntaxTree::Latex(tree) = &parent.tree {
                for include in &tree.includes {
                    for targets in &include.all_targets {
                        for target in targets {
                            if let Some(ref child) = self.find(target) {
                                graph.add_edge(
                                    indices_by_uri[&parent.uri],
                                    indices_by_uri[&child.uri],
                                    (),
                                );
                            }
                        }
                    }
                }

                if let Some(child) = Self::aux_path(&parent.uri, options)
                    .and_then(|aux_path| Uri::from_file_path(aux_path).ok())
                    .and_then(|aux_uri| self.find(&aux_uri))
                {
                    graph.add_edge(indices_by_uri[&parent.uri], indices_by_uri[&child.uri], ());
                }
            }
        }

        let mut documents = Vec::new();
        if self.find(uri).is_some() {
            let mut dfs = Dfs::new(&graph, indices_by_uri[uri]);
            while let Some(index) = dfs.next(&graph) {
                documents.push(Arc::clone(&graph.node_weight(index).unwrap()));
            }
        }
        documents
    }

    pub fn find_parent(&self, uri: &Uri, options: &Options) -> Option<Arc<Document>> {
        for document in self.related_documents(uri, options) {
            if let SyntaxTree::Latex(tree) = &document.tree {
                if tree.env.is_standalone {
                    return Some(document);
                }
            }
        }
        None
    }

    pub fn unresolved_includes(&self, options: &Options) -> Vec<PathBuf> {
        let mut includes = Vec::new();
        for document in &self.documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                for include in &tree.includes {
                    match include.kind {
                        LatexIncludeKind::Bibliography | LatexIncludeKind::Latex => (),
                        LatexIncludeKind::Everything
                        | LatexIncludeKind::Image
                        | LatexIncludeKind::Pdf
                        | LatexIncludeKind::Svg => continue,
                        LatexIncludeKind::Package | LatexIncludeKind::Class => {
                            if include
                                .paths()
                                .iter()
                                .all(|name| COMPONENT_DATABASE.contains(name.text()))
                            {
                                continue;
                            }
                        }
                    }

                    for targets in &include.all_targets {
                        if targets.iter().any(|target| self.find(target).is_some()) {
                            continue;
                        }

                        for target in targets {
                            if let Ok(path) = target.to_file_path() {
                                if path.exists() {
                                    includes.push(path);
                                }
                            }
                        }
                    }
                }

                if let Some(aux_path) = Self::aux_path(&document.uri, options) {
                    if self
                        .find(&Uri::from_file_path(&aux_path).unwrap())
                        .is_none()
                    {
                        includes.push(aux_path);
                    }
                }
            }
        }
        includes
    }

    fn aux_path(tex_uri: &Uri, options: &Options) -> Option<PathBuf> {
        let tex_path = tex_uri.to_file_path().ok()?;
        let aux_path = PathBuf::from(
            options
                .resolve_output_file(&tex_path, "aux")?
                .to_str()?
                .replace('\\', "/"),
        )
        .clean();
        Some(aux_path)
    }
}

#[derive(Debug, Default)]
pub struct TestWorkspaceBuilder {
    pub workspace: Workspace,
}

impl TestWorkspaceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_document(&mut self, name: &str, text: &str) -> Uri {
        let resolver = Resolver::default();
        let options = Options::default();
        let path = env::temp_dir().join(name);
        let language = Language::by_extension(path.extension().unwrap().to_str().unwrap()).unwrap();
        let uri = Uri::from_file_path(path).unwrap();
        let document = Document::parse(uri.clone(), text.to_owned(), language, &options, &resolver);
        self.workspace.documents.push(Arc::new(document));
        uri
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_documents(expected: Vec<Uri>, actual: Vec<Arc<Document>>) {
        assert_eq!(expected.len(), actual.len());
        for i in 0..expected.len() {
            assert_eq!(expected[i], actual[i].uri);
        }
    }

    #[test]
    fn related_documents_append_extensions() {
        let mut builder = TestWorkspaceBuilder::new();
        let uri1 = builder.add_document("foo.tex", "\\include{bar/baz}");
        let uri2 = builder.add_document("bar/baz.tex", "");
        let documents = builder
            .workspace
            .related_documents(&uri1, &Options::default());
        verify_documents(vec![uri1, uri2], documents);
    }

    #[test]
    fn related_documents_relative_path() {
        let mut builder = TestWorkspaceBuilder::new();
        let uri1 = builder.add_document("foo.tex", "");
        let uri2 = builder.add_document("bar/baz.tex", "\\input{../foo.tex}");
        let documents = builder
            .workspace
            .related_documents(&uri1, &Options::default());
        verify_documents(vec![uri1, uri2], documents);
    }

    #[test]
    fn related_documents_invalid_includes() {
        let mut builder = TestWorkspaceBuilder::new();
        let uri = builder.add_document("foo.tex", "\\include{<foo>?|bar|:}\n\\include{}");
        let documents = builder
            .workspace
            .related_documents(&uri, &Options::default());
        verify_documents(vec![uri], documents);
    }

    #[test]
    fn related_documents_bibliographies() {
        let mut builder = TestWorkspaceBuilder::new();
        let uri1 = builder.add_document("foo.tex", "\\addbibresource{bar.bib}");
        let uri2 = builder.add_document("bar.bib", "");
        let documents = builder
            .workspace
            .related_documents(&uri2, &Options::default());
        verify_documents(vec![uri2, uri1], documents);
    }

    #[test]
    fn related_documents_unresolvable_include() {
        let mut builder = TestWorkspaceBuilder::new();
        let uri = builder.add_document("foo.tex", "\\include{bar.tex}");
        builder.add_document("baz.tex", "");
        let documents = builder
            .workspace
            .related_documents(&uri, &Options::default());
        verify_documents(vec![uri], documents);
    }

    #[test]
    fn related_documents_include_cycles() {
        let mut builder = TestWorkspaceBuilder::new();
        let uri1 = builder.add_document("foo.tex", "\\input{bar.tex}");
        let uri2 = builder.add_document("bar.tex", "\\input{foo.tex}");
        let documents = builder
            .workspace
            .related_documents(&uri1, &Options::default());
        verify_documents(vec![uri1, uri2], documents);
    }

    #[test]
    fn related_documents_same_parent() {
        let mut builder = TestWorkspaceBuilder::new();
        let uri1 = builder.add_document("test.tex", "\\include{test1}\\include{test2}");
        let uri2 = builder.add_document("test1.tex", "\\label{foo}");
        let uri3 = builder.add_document("test2.tex", "\\ref{foo}");
        let documents = builder
            .workspace
            .related_documents(&uri3, &Options::default());
        verify_documents(vec![uri3, uri1, uri2], documents);
    }

    #[test]
    fn related_documents_aux_file() {
        let mut builder = TestWorkspaceBuilder::new();
        let uri1 = builder.add_document("foo.tex", "\\include{bar}");
        let uri2 = builder.add_document("bar.tex", "");
        let uri3 = builder.add_document("foo.aux", "");
        let documents = builder
            .workspace
            .related_documents(&uri2, &Options::default());
        verify_documents(vec![uri2, uri1, uri3], documents);
    }

    #[test]
    fn related_documents_aux_file_sub_directory() {
        let mut builder = TestWorkspaceBuilder::new();
        let uri1 = builder.add_document("foo.tex", "");
        let uri2 = builder.add_document("bar/baz/foo.aux", "");
        let options = Options {
            latex: Some(LatexOptions {
                build: Some(LatexBuildOptions {
                    output_directory: Some(PathBuf::from("bar/baz")),
                    ..LatexBuildOptions::default()
                }),
                ..LatexOptions::default()
            }),
            bibtex: None,
        };
        let documents = builder.workspace.related_documents(&uri1, &options);
        verify_documents(vec![uri1, uri2], documents);
    }

    #[test]
    fn find_parent() {
        let mut builder = TestWorkspaceBuilder::new();
        let uri1 = builder.add_document("foo.tex", "");
        let uri2 =
            builder.add_document("bar.tex", "\\begin{document}\\include{foo}\\end{document}");
        let document = builder
            .workspace
            .find_parent(&uri1, &Options::default())
            .unwrap();
        assert_eq!(uri2, document.uri);
    }

    #[test]
    fn find_parent_no_parent() {
        let mut builder = TestWorkspaceBuilder::new();
        let uri = builder.add_document("foo.tex", "");
        builder.add_document("bar.tex", "\\begin{document}\\end{document}");
        let document = builder.workspace.find_parent(&uri, &Options::default());
        assert_eq!(None, document);
    }
}
