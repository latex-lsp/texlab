use crate::client::LspClientMock;
use crate::server::LatexLspServer;
use copy_dir::copy_dir;
use jsonrpc::server::ActionHandler;
use lsp_types::*;
use std::fs::remove_dir;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

pub static FULL_CAPABILITIES: ClientCapabilities = ClientCapabilities {
    workspace: Some(WorkspaceClientCapabilities {
        configuration: Some(true),
        did_change_watched_files: Some(GenericCapability {
            dynamic_registration: Some(true),
        }),
        workspace_folders: None,
        apply_edit: None,
        execute_command: None,
        symbol: None,
        workspace_edit: None,
        did_change_configuration: None,
    }),
    text_document: None,
    experimental: None,
    window: Some(WindowClientCapabilities {
        progress: Some(true),
    }),
};

pub struct Scenario {
    pub server: LatexLspServer<LspClientMock>,
    pub client: Arc<LspClientMock>,
    pub directory: TempDir,
}

impl Scenario {
    pub async fn new<'a>(name: &'a str, client_capabilities: &'a ClientCapabilities) -> Self {
        let directory = tempfile::tempdir().unwrap();
        remove_dir(directory.path()).unwrap();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("scenarios")
            .join(name);
        copy_dir(source, directory.path()).unwrap();

        let client = Arc::new(LspClientMock::default());
        let server = LatexLspServer::new(Arc::clone(&client));

        let root_uri = Uri::from_file_path(directory.path()).unwrap();
        let init_params = InitializeParams {
            process_id: None,
            root_path: Some(directory.path().to_string_lossy().into_owned()),
            root_uri: Some(root_uri),
            initialization_options: None,
            capabilities: client_capabilities.to_owned(),
            trace: None,
            workspace_folders: None,
        };
        server.initialize(init_params).await.unwrap();
        server.execute_actions().await;
        server.initialized(InitializedParams {});
        server.execute_actions().await;

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
        std::fs::read_to_string(path).unwrap().replace('\r', "")
    }

    pub async fn open(&self, name: &'static str) {
        let text = self.read(name).await;
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
        });
        self.server.execute_actions().await;
    }
}
