use std::{path::PathBuf, sync::Arc};

use lsp_types::{ClientCapabilities, ClientInfo};

use crate::{
    distro::{DistroKind, Resolver},
    protocol::{Options, Uri},
    DocumentLanguage,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct Document(salsa::InternId);

impl salsa::InternKey for Document {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DocumentData {
    pub uri: Arc<Uri>,
}

#[salsa::query_group(DocumentDatabaseStorage)]
pub trait DocumentDatabase: salsa::Database {
    #[salsa::input]
    fn client_capabilities(&self) -> ClientCapabilities;

    #[salsa::input]
    fn client_info(&self) -> Option<ClientInfo>;

    #[salsa::input]
    fn current_dir(&self) -> Arc<PathBuf>;

    #[salsa::input]
    fn distro_kind(&self) -> DistroKind;

    #[salsa::input]
    fn resolver(&self) -> Arc<Resolver>;

    #[salsa::input]
    fn options(&self) -> Arc<Options>;

    #[salsa::input]
    fn all_documents(&self) -> Arc<Vec<Document>>;

    #[salsa::input]
    fn is_open(&self, document: Document) -> bool;

    #[salsa::input]
    fn text(&self, document: Document) -> Arc<String>;

    #[salsa::input]
    fn language(&self, document: Document) -> DocumentLanguage;

    #[salsa::interned]
    fn intern_document(&self, document: DocumentData) -> Document;
}
