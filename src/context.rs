use std::{
    path::PathBuf,
    sync::{Mutex, RwLock},
};

use lsp_types::{ClientCapabilities, ClientInfo};

use crate::{
    distro::{DistributionKind, Resolver},
    Options,
};

#[derive(Debug)]
pub struct ServerContext {
    pub current_directory: PathBuf,
    pub distro_kind: Mutex<DistributionKind>,
    pub resolver: Mutex<Resolver>,
    pub client_capabilities: Mutex<ClientCapabilities>,
    pub client_info: Mutex<Option<ClientInfo>>,
    pub options: RwLock<Options>,
}

impl ServerContext {
    pub fn new(current_dir: PathBuf) -> Self {
        Self {
            current_directory: current_dir,
            distro_kind: Mutex::new(DistributionKind::Unknown),
            resolver: Mutex::new(Resolver::default()),
            client_capabilities: Mutex::default(),
            client_info: Mutex::default(),
            options: RwLock::default(),
        }
    }
}
