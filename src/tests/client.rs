use std::sync::Once;

use anyhow::{bail, Result};
use crossbeam_channel::{Receiver, Sender};
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    notification::{Exit, Initialized},
    request::{Initialize, Shutdown},
    ClientCapabilities, ClientInfo, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
    InitializedParams, Url, WorkspaceFolder,
};
use tempfile::{tempdir, TempDir};

use crate::Server;

static INIT_LOGGER: Once = Once::new();

pub struct IncomingHandler {
    _handle: jod_thread::JoinHandle<Result<()>>,
    pub requests: Receiver<Request>,
    pub notifications: Receiver<Notification>,
    pub responses: Receiver<Response>,
}

impl IncomingHandler {
    pub fn spawn(receiver: Receiver<Message>) -> Self {
        let (req_sender, req_receiver) = crossbeam_channel::unbounded();
        let (not_sender, not_receiver) = crossbeam_channel::unbounded();
        let (res_sender, res_receiver) = crossbeam_channel::unbounded();

        let _handle = jod_thread::spawn(move || {
            for message in &receiver {
                match message {
                    Message::Request(req) => req_sender.send(req)?,
                    Message::Response(res) => res_sender.send(res)?,
                    Message::Notification(not) => not_sender.send(not)?,
                };
            }

            Ok(())
        });

        Self {
            _handle,
            requests: req_receiver,
            notifications: not_receiver,
            responses: res_receiver,
        }
    }
}

pub struct ClientResult {
    pub directory: TempDir,
    pub incoming: IncomingHandler,
}

pub struct Client {
    outgoing: Sender<Message>,
    incoming: IncomingHandler,
    directory: TempDir,
    request_id: i32,
    _handle: jod_thread::JoinHandle,
}

impl Client {
    pub fn spawn() -> Self {
        INIT_LOGGER.call_once(|| env_logger::init());

        let directory = tempdir().unwrap();
        let (client, server) = Connection::memory();
        let incoming = IncomingHandler::spawn(client.receiver);
        let outgoing = client.sender;
        let server = Server::new(server);
        let _handle = jod_thread::spawn(move || {
            server.run().expect("server failed to run");
        });

        Self {
            outgoing,
            incoming,
            directory,
            request_id: 0,
            _handle,
        }
    }

    #[allow(deprecated)]
    pub fn initialize(
        &mut self,
        client_capabilities: ClientCapabilities,
        client_info: Option<ClientInfo>,
    ) -> InitializeResult {
        let result = self
            .request::<Initialize>(InitializeParams {
                process_id: None,
                root_path: None,
                root_uri: None,
                initialization_options: Some(serde_json::json!({ "skipDistro": true })),
                capabilities: client_capabilities,
                trace: None,
                workspace_folders: Some(vec![WorkspaceFolder {
                    name: "Test".into(),
                    uri: Url::from_directory_path(self.directory.path()).unwrap(),
                }]),
                client_info,
                locale: None,
            })
            .unwrap();

        self.notify::<Initialized>(InitializedParams {});
        result
    }

    pub fn request<R: lsp_types::request::Request>(
        &mut self,
        params: R::Params,
    ) -> Result<R::Result> {
        self.request_id += 1;

        self.outgoing
            .send(Request::new(self.request_id.into(), R::METHOD.into(), params).into())
            .unwrap();

        let response = self.incoming.responses.recv().unwrap();
        assert_eq!(response.id, self.request_id.into());

        let result = match response.result {
            Some(result) => result,
            None => bail!("request failed: {:?}", response.error),
        };

        Ok(serde_json::from_value(result)?)
    }

    pub fn notify<N: lsp_types::notification::Notification>(&mut self, params: N::Params) {
        self.outgoing
            .send(Notification::new(N::METHOD.into(), serde_json::to_value(params).unwrap()).into())
            .unwrap();
    }

    pub fn open(&mut self, name: &str, language_id: &str, text: String) {
        self.notify::<lsp_types::notification::DidOpenTextDocument>(DidOpenTextDocumentParams {
            text_document: lsp_types::TextDocumentItem {
                uri: self.uri(name),
                language_id: language_id.to_string(),
                version: 0,
                text,
            },
        });
    }

    pub fn store_on_disk(&mut self, name: &str, text: &str) {
        let path = self.directory.path().join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    pub fn shutdown(mut self) -> ClientResult {
        self.request::<Shutdown>(()).unwrap();
        self.notify::<Exit>(());
        ClientResult {
            directory: self.directory,
            incoming: self.incoming,
        }
    }

    pub fn uri(&self, name: &str) -> Url {
        Url::from_file_path(self.directory.path().join(name)).unwrap()
    }
}
