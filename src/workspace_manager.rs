use futures::executor::block_on;
use log::*;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use texlab_distro::{Distribution, Language};
use texlab_protocol::{Options, TextDocumentItem, Uri};
use texlab_syntax::SyntaxTree;
use texlab_workspace::{Document, Workspace};

#[derive(Debug)]
pub enum WorkspaceLoadError {
    UnknownLanguage,
    InvalidPath,
    IO(std::io::Error),
}

pub struct WorkspaceManager {
    distribution: Arc<Box<dyn Distribution>>,
    workspace: Mutex<Arc<Workspace>>,
}

impl WorkspaceManager {
    pub fn new(distribution: Arc<Box<dyn Distribution>>) -> Self {
        Self {
            distribution,
            workspace: Mutex::default(),
        }
    }

    pub fn get(&self) -> Arc<Workspace> {
        let workspace = self.workspace.lock().unwrap();
        Arc::clone(&workspace)
    }

    pub fn add(&self, document: TextDocumentItem, options: &Options) {
        let language = match Language::by_language_id(&document.language_id) {
            Some(language) => language,
            None => {
                error!("Invalid language id: {}", &document.language_id);
                return;
            }
        };

        let mut workspace = self.workspace.lock().unwrap();
        *workspace = self.add_or_update(
            &workspace,
            document.uri.into(),
            document.text,
            language,
            options,
        );
    }

    pub fn load(&self, path: &Path, options: &Options) -> Result<(), WorkspaceLoadError> {
        let language = match path
            .extension()
            .and_then(OsStr::to_str)
            .and_then(Language::by_extension)
        {
            Some(language) => language,
            None => {
                warn!("Could not determine language: {}", path.to_string_lossy());
                return Err(WorkspaceLoadError::UnknownLanguage);
            }
        };

        let uri = match Uri::from_file_path(path) {
            Ok(uri) => uri,
            Err(_) => {
                error!("Invalid path: {}", path.to_string_lossy());
                return Err(WorkspaceLoadError::InvalidPath);
            }
        };

        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(why) => {
                warn!("Could not open file: {}", path.to_string_lossy());
                return Err(WorkspaceLoadError::IO(why));
            }
        };

        let mut workspace = self.workspace.lock().unwrap();
        *workspace = self.add_or_update(&workspace, uri, text, language, options);
        Ok(())
    }

    pub fn update(&self, uri: Uri, text: String, options: &Options) {
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

        *workspace = self.add_or_update(&workspace, uri, text, language, options);
    }

    fn add_or_update(
        &self,
        workspace: &Workspace,
        uri: Uri,
        text: String,
        language: Language,
        options: &Options,
    ) -> Arc<Workspace> {
        let resolver = block_on(self.distribution.resolver());
        let document = Document::parse(uri, text, language, &options, &resolver);
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
