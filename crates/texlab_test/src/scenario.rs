use super::client::MockLspClient;
use copy_dir::copy_dir;
use futures::lock::Mutex;
use once_cell::sync::Lazy;
use std::fs::remove_dir;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::{tempdir, TempDir};
use texlab::server::LatexLspServer;
use texlab_distro::{Distribution, UnknownDistribution};
use texlab_protocol::*;

static DISTRIBUTION: Lazy<Mutex<Option<Arc<Box<dyn Distribution>>>>> =
    Lazy::new(|| Mutex::new(None));

pub struct Scenario {
    pub distribution: Arc<Box<dyn Distribution>>,
    pub directory: TempDir,
    pub server: LatexLspServer<MockLspClient>,
    pub client: Arc<MockLspClient>,
}

impl Scenario {
    pub async fn new(name: &str, use_distribution: bool) -> Self {
        let distribution: Arc<Box<dyn Distribution>> = if use_distribution {
            let mut guard = DISTRIBUTION.lock().await;
            if guard.is_none() {
                *guard = Some(Arc::new(Distribution::detect().await));
            }
            Arc::clone(guard.as_ref().unwrap())
        } else {
            Arc::new(Box::new(UnknownDistribution::new()))
        };

        let directory = tempdir().unwrap();
        remove_dir(directory.path()).unwrap();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("scenarios")
            .join(name);
        copy_dir(source, directory.path()).unwrap();

        let client = Arc::new(MockLspClient::new());
        let server = LatexLspServer::new(Arc::clone(&client), Arc::clone(&distribution));
        Self {
            distribution,
            directory,
            server,
            client,
        }
    }

    pub async fn initialize(&self, capabilities: &ClientCapabilities) {
        let root_uri = Uri::from_file_path(self.directory.path()).unwrap();
        let params = InitializeParams {
            process_id: None,
            root_path: Some(self.directory.path().to_string_lossy().into_owned()),
            root_uri: Some(root_uri.into()),
            initialization_options: None,
            capabilities: capabilities.clone(),
            trace: None,
            workspace_folders: None,
        };

        self.server
            .execute(|svr| svr.initialize(params))
            .await
            .unwrap();

        self.server
            .execute(|svr| svr.initialized(InitializedParams {}))
            .await;
    }

    pub fn uri(&self, name: &str) -> Uri {
        let mut path = self.directory.path().to_owned();
        path.push(name);
        Uri::from_file_path(path).unwrap()
    }

    pub async fn read(&self, name: &'static str) -> String {
        let mut path = self.directory.path().to_owned();
        path.push(name);
        let data = tokio::fs::read(path)
            .await
            .expect("failed to read scenario file");
        let text = String::from_utf8_lossy(&data);
        text.replace('\r', "")
    }

    pub async fn open(&self, name: &'static str) {
        let text = self.read(name).await;
        let language_id = if name.ends_with(".bib") {
            "bibtex"
        } else {
            "latex"
        };

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: self.uri(name).into(),
                version: 0,
                language_id: language_id.to_owned(),
                text,
            },
        };
        self.server.execute(|svr| svr.did_open(params)).await;
    }
}
