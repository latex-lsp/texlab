#![feature(await_macro, async_await)]

use copy_dir::copy_dir;
use futures::future::BoxFuture;
use futures::lock::Mutex;
use futures::prelude::*;
use jsonrpc::client::FutureResult;
use jsonrpc::server::EventHandler;
use lsp_types::*;
use std::borrow::Cow;
use std::fs::remove_dir;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use texlab::client::LspClient;
use texlab::formatting::bibtex::BibtexFormattingOptions;
use texlab::server::LatexLspServer;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LspClientMockOptions {
    pub bibtex_formatting: Option<BibtexFormattingOptions>,
}

#[derive(Debug, Default)]
pub struct LspClientMock {
    pub messages: Mutex<Vec<ShowMessageParams>>,
    pub options: Mutex<LspClientMockOptions>,
}

impl LspClientMock {
    pub async fn set_bibtex_formatting(&self, bibtex_formatting: BibtexFormattingOptions) {
        let mut options = await!(self.options.lock());
        options.bibtex_formatting = Some(bibtex_formatting);
    }
}

impl LspClient for LspClientMock {
    fn configuration(&self, params: ConfigurationParams) -> FutureResult<'_, serde_json::Value> {
        let handler = async move {
            let options = await!(self.options.lock());
            match params.items[0].section {
                Some(Cow::Borrowed("bibtex.formatting")) => {
                    let error = jsonrpc::Error {
                        code: jsonrpc::ErrorCode::InternalError,
                        message: "Internal error".to_owned(),
                        data: serde_json::Value::Null,
                    };

                    options
                        .bibtex_formatting
                        .as_ref()
                        .map(|options| serde_json::to_value(vec![options]).unwrap())
                        .ok_or(error)
                }
                _ => {
                    unreachable!();
                }
            }
        };
        handler.boxed()
    }

    fn show_message(&self, params: ShowMessageParams) -> BoxFuture<'_, ()> {
        let handler = async move {
            let mut messages = await!(self.messages.lock());
            messages.push(params);
        };
        handler.boxed()
    }

    fn register_capability(&self, _params: RegistrationParams) -> FutureResult<'_, ()> {
        let handler = async move { Ok(()) };
        handler.boxed()
    }

    fn publish_diagnostics(&self, _params: PublishDiagnosticsParams) -> BoxFuture<'_, ()> {
        let handler = async move {};
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
