use std::{
    fs,
    sync::{
        atomic::{AtomicI32, Ordering},
        Mutex,
    },
    thread::{self, JoinHandle},
};

use anyhow::Result;
use lsp_server::{Connection, RequestId};
use lsp_types::{notification::Notification, request::Request, *};
use rustc_hash::FxHashMap;
use tempfile::{tempdir, TempDir};
use texlab::Server;
use unindent::unindent;

pub struct ServerTester {
    pub directory: TempDir,
    client: Connection,
    handle: Option<JoinHandle<()>>,
    request_id: AtomicI32,
    pub diagnostics_by_uri: Mutex<FxHashMap<Url, Vec<Diagnostic>>>,
}

impl ServerTester {
    pub fn launch_new_instance() -> Result<Self> {
        let directory = tempdir()?;
        let (conn, client) = Connection::memory();
        let server = Server::with_connection(conn, directory.path().to_path_buf(), false)?;
        let handle = thread::spawn(move || server.run().unwrap());
        Ok(Self {
            directory,
            client,
            handle: Some(handle),
            request_id: AtomicI32::new(0),
            diagnostics_by_uri: Mutex::default(),
        })
    }

    fn wait_for_response(&self, request_id: RequestId) -> Result<lsp_server::Response> {
        loop {
            match self.client.receiver.recv()? {
                lsp_server::Message::Request(request) => {
                    match request.method.as_str() {
                        request::RegisterCapability::METHOD => {
                            self.client
                                .sender
                                .send(lsp_server::Response::new_ok(request.id, ()).into())?;
                        }
                        method => {
                            panic!("unknown request: {}", method);
                        }
                    };
                }
                lsp_server::Message::Notification(notification) => {
                    match notification.method.as_str() {
                        notification::PublishDiagnostics::METHOD => {
                            let params = serde_json::from_value::<PublishDiagnosticsParams>(
                                notification.params,
                            )?;
                            let mut diagnostics_by_uri = self.diagnostics_by_uri.lock().unwrap();
                            diagnostics_by_uri.insert(params.uri, params.diagnostics);
                        }
                        method => {
                            panic!("unknown notification: {}", method);
                        }
                    };
                }
                lsp_server::Message::Response(response) => {
                    assert_eq!(response.id, request_id);
                    return Ok(response);
                }
            }
        }
    }

    #[allow(deprecated)]
    pub fn initialize(
        &self,
        client_capabilities: ClientCapabilities,
        client_info: Option<ClientInfo>,
    ) -> anyhow::Result<()> {
        let request_id = RequestId::from(self.request_id.fetch_add(1, Ordering::SeqCst));
        self.client.sender.send(
            lsp_server::Request::new(
                request_id.clone(),
                request::Initialize::METHOD.to_string(),
                InitializeParams {
                    process_id: None,
                    root_path: None,
                    root_uri: None,
                    initialization_options: None,
                    capabilities: client_capabilities,
                    trace: None,
                    workspace_folders: None,
                    client_info,
                    locale: None,
                },
            )
            .into(),
        )?;
        self.wait_for_response(request_id)?;

        self.client.sender.send(
            lsp_server::Notification::new(notification::Initialized::METHOD.to_string(), ()).into(),
        )?;

        Ok(())
    }

    pub fn open(&self, name: &str, text: &str, language_id: &str, store: bool) -> Result<Url> {
        let text = unindent(text).trim().to_string();
        let path = self.directory.path().join(name);
        if store {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, &text)?;
        }

        let uri = Url::from_file_path(path).unwrap();
        self.client.sender.send(
            lsp_server::Notification::new(
                notification::DidOpenTextDocument::METHOD.to_string(),
                DidOpenTextDocumentParams {
                    text_document: TextDocumentItem::new(
                        uri.clone(),
                        language_id.to_string(),
                        0,
                        text,
                    ),
                },
            )
            .into(),
        )?;
        Ok(uri)
    }

    pub fn open_memory(&self, uri: Url, text: &str, language_id: &str) -> Result<()> {
        let text = unindent(text).trim().to_string();
        let text_document = TextDocumentItem::new(uri, language_id.to_string(), 0, text);
        self.client.sender.send(
            lsp_server::Notification::new(
                notification::DidOpenTextDocument::METHOD.to_string(),
                DidOpenTextDocumentParams { text_document },
            )
            .into(),
        )?;
        Ok(())
    }

    // pub fn edit(&self, uri: Url, text: &str) -> Result<()> {
    //     let text = unindent(text).trim().to_string();
    //     self.client.sender.send(
    //         lsp_server::Notification::new(
    //             notification::DidChangeTextDocument::METHOD.to_string(),
    //             DidChangeTextDocumentParams {
    //                 text_document: VersionedTextDocumentIdentifier::new(uri, 0),
    //                 content_changes: vec![TextDocumentContentChangeEvent {
    //                     text,
    //                     range: None,
    //                     range_length: None,
    //                 }],
    //             },
    //         )
    //         .into(),
    //     )?;
    //     Ok(())
    // }

    pub fn complete(&self, uri: Url, line: u32, character: u32) -> Result<CompletionList> {
        let request_id = RequestId::from(self.request_id.fetch_add(1, Ordering::SeqCst));

        self.client.sender.send(
            lsp_server::Request::new(
                request_id.clone(),
                request::Completion::METHOD.to_string(),
                CompletionParams {
                    text_document_position: TextDocumentPositionParams::new(
                        TextDocumentIdentifier::new(uri),
                        Position::new(line, character),
                    ),
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                    context: None,
                },
            )
            .into(),
        )?;

        let response = self.wait_for_response(request_id)?;
        let list = serde_json::from_value(response.result.expect("completion request failed"))?;
        Ok(list)
    }

    pub fn resolve_completion_item(&self, item: CompletionItem) -> Result<CompletionItem> {
        let request_id = RequestId::from(self.request_id.fetch_add(1, Ordering::SeqCst));

        self.client.sender.send(
            lsp_server::Request::new(
                request_id.clone(),
                request::ResolveCompletionItem::METHOD.to_string(),
                item,
            )
            .into(),
        )?;

        let response = self.wait_for_response(request_id)?;
        let result = serde_json::from_value(
            response
                .result
                .expect("resolve completion item request failed"),
        )?;
        Ok(result)
    }

    pub fn hover(&self, uri: Url, line: u32, character: u32) -> Result<Option<Hover>> {
        let request_id = RequestId::from(self.request_id.fetch_add(1, Ordering::SeqCst));

        self.client.sender.send(
            lsp_server::Request::new(
                request_id.clone(),
                request::HoverRequest::METHOD.to_string(),
                HoverParams {
                    text_document_position_params: TextDocumentPositionParams::new(
                        TextDocumentIdentifier::new(uri),
                        Position::new(line, character),
                    ),
                    work_done_progress_params: WorkDoneProgressParams::default(),
                },
            )
            .into(),
        )?;

        let response = self.wait_for_response(request_id)?;
        let hover = serde_json::from_value(response.result.expect("hover request failed"))?;
        Ok(hover)
    }

    pub fn find_document_symbols(&self, uri: Url) -> Result<DocumentSymbolResponse> {
        let request_id = RequestId::from(self.request_id.fetch_add(1, Ordering::SeqCst));

        self.client.sender.send(
            lsp_server::Request::new(
                request_id.clone(),
                request::DocumentSymbolRequest::METHOD.to_string(),
                DocumentSymbolParams {
                    text_document: TextDocumentIdentifier::new(uri),
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                },
            )
            .into(),
        )?;

        let response = self.wait_for_response(request_id)?;
        let symbols =
            serde_json::from_value(response.result.expect("document symbol request failed"))?;
        Ok(symbols)
    }

    pub fn find_workspace_symbols(&self, query: &str) -> Result<Vec<SymbolInformation>> {
        let request_id = RequestId::from(self.request_id.fetch_add(1, Ordering::SeqCst));

        self.client.sender.send(
            lsp_server::Request::new(
                request_id.clone(),
                request::WorkspaceSymbol::METHOD.to_string(),
                WorkspaceSymbolParams {
                    partial_result_params: PartialResultParams::default(),
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    query: query.to_string(),
                },
            )
            .into(),
        )?;

        let response = self.wait_for_response(request_id)?;
        let symbols =
            serde_json::from_value(response.result.expect("workspace symbol request failed"))?;
        Ok(symbols)
    }

    // pub fn change_configuration(&self, options: Options) -> Result<()> {
    //     self.client.sender.send(
    //         lsp_server::Notification::new(
    //             notification::DidChangeConfiguration::METHOD.to_string(),
    //             DidChangeConfigurationParams {
    //                 settings: serde_json::to_value(options)?,
    //             },
    //         )
    //         .into(),
    //     )?;
    //     Ok(())
    // }
}

impl Drop for ServerTester {
    fn drop(&mut self) {
        self.client
            .sender
            .send(
                lsp_server::Request::new(
                    RequestId::from(self.request_id.fetch_add(1, Ordering::SeqCst)),
                    request::Shutdown::METHOD.to_string(),
                    (),
                )
                .into(),
            )
            .unwrap();

        self.client
            .sender
            .send(lsp_server::Notification::new(notification::Exit::METHOD.to_string(), ()).into())
            .unwrap();

        self.handle.take().unwrap().join().unwrap();
    }
}
