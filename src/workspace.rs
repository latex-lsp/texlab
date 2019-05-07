use crate::syntax::bibtex::BibtexSyntaxTree;
use crate::syntax::latex::*;
use log::*;
use lsp_types::TextDocumentItem;
use path_clean::PathClean;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use url::Url;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Language {
    Latex,
    Bibtex,
}

impl Language {
    pub fn by_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_ref() {
            "tex" | "sty" | "cls" => Some(Language::Latex),
            "bib" => Some(Language::Bibtex),
            _ => None,
        }
    }

    pub fn by_language_id(language_id: &str) -> Option<Self> {
        match language_id {
            "latex" => Some(Language::Latex),
            "bibtex" => Some(Language::Bibtex),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SyntaxTree {
    Latex(LatexSyntaxTree),
    Bibtex(BibtexSyntaxTree),
}

impl SyntaxTree {
    pub fn parse(text: &str, language: Language) -> Self {
        match language {
            Language::Latex => SyntaxTree::Latex(LatexSyntaxTree::from(text)),
            Language::Bibtex => SyntaxTree::Bibtex(BibtexSyntaxTree::from(text)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Document {
    pub uri: Url,
    pub text: String,
    pub tree: SyntaxTree,
}

impl Document {
    pub fn new(uri: Url, text: String, tree: SyntaxTree) -> Self {
        Document { uri, text, tree }
    }

    pub fn parse(uri: Url, text: String, language: Language) -> Self {
        let tree = SyntaxTree::parse(&text, language);
        Document::new(uri, text, tree)
    }

    pub fn is_file(&self) -> bool {
        self.uri.scheme() == "file"
    }
}

const DOCUMENT_EXTENSIONS: &'static [&'static str] = &[".tex", ".sty", ".cls", ".bib"];

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

    pub fn find(&self, uri: &Url) -> Option<Arc<Document>> {
        self.documents
            .iter()
            .find(|document| document.uri == *uri)
            .map(|document| Arc::clone(&document))
    }

    pub fn resolve_document(&self, uri: &Url, relative_path: &str) -> Option<Arc<Document>> {
        let targets = resolve_link_targets(uri, relative_path)?;
        for target in targets {
            if let Ok(target_uri) = Url::from_file_path(target) {
                if let Some(document) = self.find(&target_uri) {
                    if document.is_file() {
                        return Some(document);
                    }
                }
            }
        }
        None
    }

    pub fn related_documents(&self, uri: &Url) -> Vec<Arc<Document>> {
        let mut edges: Vec<(Arc<Document>, Arc<Document>)> = Vec::new();
        for parent in self.documents.iter().filter(|document| document.is_file()) {
            if let SyntaxTree::Latex(tree) = &parent.tree {
                let mut analyzer = LatexIncludeAnalyzer::new();
                analyzer.visit_root(&tree.root);
                for include in analyzer.included_files {
                    if let Some(ref child) = self.resolve_document(&parent.uri, include.path.text())
                    {
                        edges.push((Arc::clone(&parent), Arc::clone(&child)));
                        edges.push((Arc::clone(&child), Arc::clone(&parent)));
                    }
                }
            }
        }

        let mut results = Vec::new();
        if let Some(start) = self.find(uri) {
            let mut visited: Vec<Arc<Document>> = Vec::new();
            let mut stack = Vec::new();
            stack.push(start);
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

    pub fn find_parent(&self, uri: &Url) -> Option<Arc<Document>> {
        for document in self.related_documents(uri) {
            if let SyntaxTree::Latex(tree) = &document.tree {
                let mut analyzer = LatexEnvironmentAnalyzer::new();
                analyzer.visit_root(&tree.root);
                let is_standalone = analyzer.environments.iter().any(|environment| {
                    environment
                        .left
                        .name
                        .map(LatexToken::text)
                        .unwrap_or_default()
                        == "document"
                });
                if is_standalone {
                    return Some(document);
                }
            }
        }
        None
    }
}

fn resolve_link_targets(uri: &Url, relative_path: &str) -> Option<Vec<String>> {
    let mut targets = Vec::new();
    if uri.scheme() != "file" {
        return None;
    }

    let mut path = uri.to_file_path().ok()?;
    path.pop();
    path.push(relative_path);
    path = PathBuf::from(path.to_string_lossy().replace("\\", "/"));
    path = path.clean();
    let path = path.to_string_lossy().into_owned();
    for extension in DOCUMENT_EXTENSIONS {
        targets.push(format!("{}{}", path, extension));
    }
    targets.push(path);
    targets.reverse();
    Some(targets)
}

pub struct WorkspaceManager {
    workspace: Mutex<Arc<Workspace>>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        WorkspaceManager {
            workspace: Mutex::new(Arc::new(Workspace::default())),
        }
    }

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

        let uri = match Url::from_file_path(path) {
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

    pub fn update(&self, uri: Url, text: String) {
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
        uri: Url,
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

#[cfg(test)]
pub struct WorkspaceBuilder {
    pub workspace: Workspace,
}

#[cfg(test)]
impl WorkspaceBuilder {
    pub fn new() -> Self {
        WorkspaceBuilder {
            workspace: Workspace::default(),
        }
    }

    pub fn document(&mut self, name: &str, text: &str) -> Url {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name);
        let language = Language::by_extension(path.extension().unwrap().to_str().unwrap()).unwrap();
        let uri = Url::from_file_path(path).unwrap();
        let document = Document::parse(uri.clone(), text.to_owned(), language);
        self.workspace.documents.push(Arc::new(document));
        uri
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_documents(expected: Vec<Url>, actual: Vec<Arc<Document>>) {
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
