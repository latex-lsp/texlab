use crate::client::LspClientMock;
use crate::server::LatexLspServer;
use copy_dir::copy_dir;
use jsonrpc::server::ActionHandler;
use lsp_types::*;
use std::fs::remove_dir;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

pub struct Scenario {
    pub server: LatexLspServer<LspClientMock>,
    pub client: Arc<LspClientMock>,
    pub directory: TempDir,
}

impl Scenario {
    pub async fn new(name: &str) -> Self {
        let client = Arc::new(LspClientMock::default());
        let server = LatexLspServer::new(Arc::clone(&client));

        let directory = tempfile::tempdir().unwrap();
        remove_dir(directory.path()).unwrap();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("scenarios")
            .join(name);
        copy_dir(source, directory.path()).unwrap();

        let root_uri = Uri::from_file_path(directory.path()).unwrap();
        let init_params = InitializeParams {
            process_id: None,
            root_path: Some(directory.path().to_string_lossy().into_owned()),
            root_uri: Some(root_uri),
            initialization_options: None,
            capabilities: ClientCapabilities::default(),
            trace: None,
            workspace_folders: None,
        };
        await!(server.initialize(init_params)).unwrap();
        await!(server.execute_actions());
        server.initialized(InitializedParams {});
        await!(server.execute_actions());

        Self {
            server,
            client,
            directory,
        }
    }

    pub fn uri(&self, name: &str) -> Uri {
        let mut path = self.directory.path().to_owned();
        path.push(name);
        Uri::from_file_path(path).unwrap()
    }

    pub async fn read(&self, name: &'static str) -> String {
        let mut path = self.directory.path().to_owned();
        path.push(name);
        std::fs::read_to_string(path).unwrap()
    }

    pub async fn open(&self, name: &'static str) {
        let text = await!(self.read(name));
        let language_id = if name.ends_with(".tex") {
            "latex"
        } else {
            "bibtex"
        };

        self.server.did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: self.uri(name),
                version: 0,
                language_id: language_id.to_owned(),
                text,
            },
        })
    }
}
