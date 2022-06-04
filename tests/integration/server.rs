use std::time::Duration;

use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    notification::{Exit, Initialized, Notification as _, PublishDiagnostics},
    request::{Initialize, Request as _, Shutdown},
    Diagnostic, PublishDiagnosticsParams, Url,
};
use rustc_hash::FxHashMap;
use serde_json::json;
use tempfile::{tempdir, TempDir};
use texlab::Server;

pub struct ServerState {
    pub directory: TempDir,
    pub diagnostics_by_uri: FxHashMap<Url, Vec<Diagnostic>>,
    pub registrations: Vec<String>,
}

pub struct ServerWrapper {
    client: Connection,
    request_id: i32,
    state: ServerState,
    _handle: jod_thread::JoinHandle,
}

impl ServerWrapper {
    pub fn spawn() -> Result<Self> {
        let (client, server) = Connection::memory();
        let server = Server::with_connection(server, std::env::temp_dir(), false);
        let handle = jod_thread::spawn(move || {
            server.run().expect("server failed to run");
        });

        let state = ServerState {
            directory: tempdir()?,
            diagnostics_by_uri: FxHashMap::default(),
            registrations: Vec::new(),
        };

        Ok(Self {
            client,
            request_id: 0,
            state,
            _handle: handle,
        })
    }

    pub fn initialize(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        let result = self.request::<Initialize>(params)?;
        self.notify::<Initialized>(json!(null))?;
        Ok(result)
    }

    pub fn request<R: lsp_types::request::Request>(
        &mut self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.request_id += 1;

        self.client
            .sender
            .send(Request::new(self.request_id.into(), R::METHOD.into(), params).into())?;

        loop {
            match self.client.receiver.recv_timeout(Duration::from_secs(15))? {
                Message::Request(request) => match request.method.as_str() {
                    lsp_types::request::RegisterCapability::METHOD => {
                        self.client
                            .sender
                            .send(Response::new_ok(request.id, json!({})).into())?;
                    }
                    _ => unreachable!(),
                },
                Message::Response(response) => {
                    assert_eq!(response.id, self.request_id.into());
                    return Ok(response
                        .result
                        .unwrap_or_else(|| response.error.map_or(json!(null), |why| json!(why))));
                }
                Message::Notification(Notification { method, params }) => match method.as_str() {
                    PublishDiagnostics::METHOD => {
                        let params: PublishDiagnosticsParams = serde_json::from_value(params)?;
                        self.state
                            .diagnostics_by_uri
                            .insert(params.uri, params.diagnostics);
                    }
                    _ => unreachable!(),
                },
            }
        }
    }

    pub fn notify<N: lsp_types::notification::Notification>(
        &mut self,
        params: serde_json::Value,
    ) -> Result<()> {
        self.client
            .sender
            .send(Notification::new(N::METHOD.into(), params).into())?;
        Ok(())
    }

    pub fn shutdown(mut self) -> Result<ServerState> {
        self.request::<Shutdown>(json!({}))?;
        self.notify::<Exit>(json!({}))?;
        Ok(self.state)
    }

    pub fn uri(&self, name: &str) -> Url {
        Url::from_file_path(self.state.directory.path().join(name)).unwrap()
    }
}
