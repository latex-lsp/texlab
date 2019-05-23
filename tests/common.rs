#![feature(await_macro, async_await)]

use copy_dir::copy_dir;
use futures::future::BoxFuture;
use futures::lock::Mutex;
use futures::prelude::*;
use jsonrpc::client::FutureResult;
use jsonrpc::server::EventHandler;
use lsp_types::*;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use texlab::client::LspClient;
use texlab::server::LatexLspServer;
use std::fs::remove_dir;

#[derive(Debug, Default)]
pub struct LspClientMock {
    pub messages: Mutex<Vec<ShowMessageParams>>,
}

impl LspClient for LspClientMock {
    fn configuration(&self, _params: ConfigurationParams) -> FutureResult<'_, serde_json::Value> {
        let handler = async move { Ok(serde_json::Value::Null) };
        handler.boxed()
    }

    fn show_message(&self, params: ShowMessageParams) -> BoxFuture<'_, ()> {
        let handler = async move {
            let mut messages = await!(self.messages.lock());
            messages.push(params);
        };
        handler.boxed()
    }
}

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
        await!(server.handle_events());
        server.initialized(InitializedParams {});
        await!(server.handle_events());

        Scenario {
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

    pub async fn open(&self, name: &'static str, language_id: &'static str) {
        let mut path = self.directory.path().to_owned();
        path.push(name);
        self.server.did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: self.uri(name),
                version: 0,
                language_id: language_id.to_owned(),
                text: std::fs::read_to_string(path).unwrap(),
            },
        })
    }
}
