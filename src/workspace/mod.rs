mod feature;
mod outline;

pub use self::feature::*;
pub use self::outline::*;
use crate::syntax::*;
use log::*;
use lsp_types::{TextDocumentItem, Uri};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Document {
    pub uri: Uri,
    pub text: String,
    pub tree: SyntaxTree,
}

impl Document {
    pub fn new(uri: Uri, text: String, tree: SyntaxTree) -> Self {
        Document { uri, text, tree }
    }

    pub fn parse(uri: Uri, text: String, language: Language) -> Self {
        let tree = SyntaxTree::parse(&uri, &text, language);
        Document::new(uri, text, tree)
    }

    pub fn is_file(&self) -> bool {
        self.uri.scheme() == "file"
    }
}

fn eq_uri(left: &Uri, right: &Uri) -> bool {
    if cfg!(windows) {
        left.as_str().to_lowercase() == right.as_str().to_lowercase()
    } else {
        left.as_str() == right.as_str()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Workspace {
    pub documents: Vec<Arc<Document>>,
}

impl Workspace {
    pub fn new() -> Self {
        Workspace {
            documents: Vec::new(),
        }
    }

    pub fn find(&self, uri: &Uri) -> Option<Arc<Document>> {
        self.documents
            .iter()
            .find(|document| eq_uri(&document.uri, uri))
            .map(|document| Arc::clone(&document))
    }

    pub fn related_documents(&self, uri: &Uri) -> Vec<Arc<Document>> {
        let mut edges: Vec<(Arc<Document>, Arc<Document>)> = Vec::new();
        for parent in self.documents.iter().filter(|document| document.is_file()) {
            if let SyntaxTree::Latex(tree) = &parent.tree {
                for include in &tree.includes {
                    for targets in &include.all_targets {
                        for target in targets {
                            if let Some(ref child) = self.find(target) {
                                edges.push((Arc::clone(&parent), Arc::clone(&child)));
                                edges.push((Arc::clone(&child), Arc::clone(&parent)));
                            }
                        }
                    }
                }
            }
        }

        let mut results = Vec::new();
        if let Some(start) = self.find(uri) {
            let mut visited: Vec<Arc<Document>> = Vec::new();
            let mut stack = vec![start];
            while let Some(current) = stack.pop() {
                if visited.contains(&current) {
                    continue;
                }
                visited.push(Arc::clone(&current));

                results.push(Arc::clone(&current));
                for edge in &edges {
                    if edge.0 == current {
                        stack.push(Arc::clone(&edge.1));
                    }
                }
            }
        }
        results
    }

    pub fn find_parent(&self, uri: &Uri) -> Option<Arc<Document>> {
        for document in self.related_documents(uri) {
            if let SyntaxTree::Latex(tree) = &document.tree {
                if tree.is_standalone {
                    return Some(document);
                }
            }
        }
        None
    }

    pub fn unresolved_includes(&self) -> Vec<PathBuf> {
        let mut includes = Vec::new();
        for document in &self.documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                for include in &tree.includes {
                    if include.kind != LatexIncludeKind::Latex
                        && include.kind != LatexIncludeKind::Bibliography
                    {
                        continue;
                    }

                    for targets in &include.all_targets {
                        if targets.iter().any(|target| self.find(target).is_some()) {
                            continue;
                        }

                        for target in targets {
                            let path = target.to_file_path().unwrap();
                            if path.exists() {
                                includes.push(path);
                            }
                        }
                    }
                }
            }
        }
        includes
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DocumentView {
    pub workspace: Arc<Workspace>,
    pub document: Arc<Document>,
    pub related_documents: Vec<Arc<Document>>,
}

impl DocumentView {
    pub fn new(workspace: Arc<Workspace>, document: Arc<Document>) -> Self {
        let related_documents = workspace.related_documents(&document.uri);
        Self {
            workspace,
            document,
            related_documents,
        }
    }
}

#[derive(Debug, Default)]
pub struct WorkspaceManager {
    workspace: Mutex<Arc<Workspace>>,
}

impl WorkspaceManager {
    pub fn get(&self) -> Arc<Workspace> {
        let workspace = self.workspace.lock().unwrap();
        Arc::clone(&workspace)
    }

    pub fn add(&self, document: TextDocumentItem) {
        let language = match Language::by_language_id(&document.language_id) {
            Some(language) => language,
            None => {
                error!("Invalid language id: {}", &document.language_id);
                return;
            }
        };

        let mut workspace = self.workspace.lock().unwrap();
        *workspace = Self::add_or_update(&workspace, document.uri, document.text, language);
    }

    pub fn load(&self, path: &Path) {
        let language = match path
            .extension()
            .and_then(OsStr::to_str)
            .and_then(Language::by_extension)
        {
            Some(language) => language,
            None => {
                warn!("Could not determine language: {}", path.to_string_lossy());
                return;
            }
        };

        let uri = match Uri::from_file_path(path) {
            Ok(uri) => uri,
            Err(_) => {
                error!("Invalid path: {}", path.to_string_lossy());
                return;
            }
        };

        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(_) => {
                warn!("Could not open file: {}", path.to_string_lossy());
                return;
            }
        };

        let mut workspace = self.workspace.lock().unwrap();
        *workspace = Self::add_or_update(&workspace, uri, text, language);
    }

    pub fn update(&self, uri: Uri, text: String) {
        let mut workspace = self.workspace.lock().unwrap();

        let old_document = match workspace.documents.iter().find(|x| x.uri == uri) {
            Some(document) => document,
            None => {
                warn!("Document not found: {}", uri);
                return;
            }
        };

        let language = match old_document.tree {
            SyntaxTree::Latex(_) => Language::Latex,
            SyntaxTree::Bibtex(_) => Language::Bibtex,
        };

        *workspace = Self::add_or_update(&workspace, uri, text, language);
    }

    fn add_or_update(
        workspace: &Workspace,
        uri: Uri,
        text: String,
        language: Language,
    ) -> Arc<Workspace> {
        let document = Document::parse(uri, text, language);
        let mut documents: Vec<Arc<Document>> = workspace
            .documents
            .iter()
            .filter(|x| x.uri != document.uri)
            .cloned()
            .collect();

        documents.push(Arc::new(document));
        Arc::new(Workspace { documents })
    }
}

pub struct WorkspaceBuilder {
    pub workspace: Workspace,
}

impl WorkspaceBuilder {
    pub fn new() -> Self {
        WorkspaceBuilder {
            workspace: Workspace::default(),
        }
    }

    pub fn document(&mut self, name: &str, text: &str) -> Uri {
        let path = std::env::temp_dir().join(name);
        let language = Language::by_extension(path.extension().unwrap().to_str().unwrap()).unwrap();
        let uri = Uri::from_file_path(path).unwrap();
        let document = Document::parse(uri.clone(), text.to_owned(), language);
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
    fn test_related_documents_append_extensions() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("foo.tex", "\\include{bar/baz}");
        let uri2 = builder.document("bar/baz.tex", "");
        let documents = builder.workspace.related_documents(&uri1);
        verify_documents(vec![uri1, uri2], documents);
    }

    #[test]
    fn test_related_documents_relative_path() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("foo.tex", "");
        let uri2 = builder.document("bar/baz.tex", "\\input{../foo.tex}");
        let documents = builder.workspace.related_documents(&uri1);
        verify_documents(vec![uri1, uri2], documents);
    }

    #[test]
    fn test_related_documents_invalid_includes() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\include{<foo>?|bar|:}\n\\include{}");
        let documents = builder.workspace.related_documents(&uri);
        verify_documents(vec![uri], documents);
    }

    #[test]
    fn test_related_documents_bibliographies() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("foo.tex", "\\addbibresource{bar.bib}");
        let uri2 = builder.document("bar.bib", "");
        let documents = builder.workspace.related_documents(&uri2);
        verify_documents(vec![uri2, uri1], documents);
    }

    #[test]
    fn test_related_documents_unresolvable_include() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\include{bar.tex}");
        builder.document("baz.tex", "");
        let documents = builder.workspace.related_documents(&uri);
        verify_documents(vec![uri], documents);
    }

    #[test]
    fn test_related_documents_include_cycles() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("foo.tex", "\\input{bar.tex}");
        let uri2 = builder.document("bar.tex", "\\input{foo.tex}");
        let documents = builder.workspace.related_documents(&uri1);
        verify_documents(vec![uri1, uri2], documents);
    }

    #[test]
    fn test_related_documents_same_parent() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("test.tex", "\\include{test1}\\include{test2}");
        let uri2 = builder.document("test1.tex", "\\label{foo}");
        let uri3 = builder.document("test2.tex", "\\ref{foo}");
        let documents = builder.workspace.related_documents(&uri3);
        verify_documents(vec![uri3, uri1, uri2], documents);
    }

    #[test]
    fn test_find_parent() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("foo.tex", "");
        let uri2 = builder.document("bar.tex", "\\begin{document}\\include{foo}\\end{document}");
        let document = builder.workspace.find_parent(&uri1).unwrap();
        assert_eq!(uri2, document.uri);
    }

    #[test]
    fn test_find_parent_no_parent() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "");
        builder.document("bar.tex", "\\begin{document}\\end{document}");
        let document = builder.workspace.find_parent(&uri);
        assert_eq!(None, document);
    }
}
