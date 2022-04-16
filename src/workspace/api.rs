use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Result;

use crate::{DocumentLanguage, Uri};

use super::Document;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum WorkspaceSource {
    Client,
    Server,
}

#[derive(Debug, Clone)]
pub struct WorkspaceSubset {
    pub documents: Vec<Arc<Document>>,
}

pub type OpenHandler = Arc<dyn Fn(Arc<dyn Workspace>, Arc<Document>) + Send + Sync + 'static>;

pub trait Workspace: Send + Sync {
    fn open(
        &self,
        uri: Arc<Uri>,
        text: String,
        language: DocumentLanguage,
        source: WorkspaceSource,
    ) -> Arc<Document>;

    fn register_open_handler(&self, handler: OpenHandler);

    fn reload(&self, path: PathBuf) -> Result<Option<Arc<Document>>> {
        let uri = Arc::new(Uri::from_file_path(path.clone()).unwrap());

        if self.is_open(&uri) && !uri.as_str().ends_with(".log") {
            return Ok(self.get(&uri));
        }

        if let Some(language) = DocumentLanguage::by_path(&path) {
            let data = fs::read(&path)?;
            let text = String::from_utf8_lossy(&data).into_owned();
            Ok(Some(self.open(
                uri,
                text,
                language,
                WorkspaceSource::Server,
            )))
        } else {
            Ok(None)
        }
    }

    fn load(&self, path: PathBuf) -> Result<Option<Arc<Document>>> {
        let uri = Arc::new(Uri::from_file_path(path.clone()).unwrap());

        if let Some(document) = self.get(&uri) {
            return Ok(Some(document));
        }

        let data = fs::read(&path)?;
        let text = String::from_utf8_lossy(&data).into_owned();
        if let Some(language) = DocumentLanguage::by_path(&path) {
            Ok(Some(self.open(
                uri,
                text,
                language,
                WorkspaceSource::Server,
            )))
        } else {
            Ok(None)
        }
    }

    fn documents(&self) -> Vec<Arc<Document>>;

    fn has(&self, uri: &Uri) -> bool;

    fn get(&self, uri: &Uri) -> Option<Arc<Document>>;

    fn close(&self, uri: &Uri);

    fn delete(&self, uri: &Uri);

    fn is_open(&self, uri: &Uri) -> bool;

    fn subset(&self, uri: Arc<Uri>) -> Option<WorkspaceSubset>;
}
