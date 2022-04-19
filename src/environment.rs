use std::{path::PathBuf, sync::Arc};

use lsp_types::{ClientCapabilities, ClientInfo};

use crate::{distro::Resolver, Options};

#[derive(Debug, Clone)]
pub struct Environment {
    pub current_directory: Arc<PathBuf>,
    pub client_capabilities: Arc<ClientCapabilities>,
    pub client_info: Option<Arc<ClientInfo>>,
    pub options: Arc<Options>,
    pub resolver: Arc<Resolver>,
}

impl Environment {
    pub fn new(current_directory: Arc<PathBuf>) -> Self {
        Self {
            current_directory,
            client_capabilities: Arc::new(ClientCapabilities::default()),
            client_info: None,
            options: Arc::new(Options::default()),
            resolver: Arc::new(Resolver::default()),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(Arc::new(std::env::temp_dir()))
    }
}
