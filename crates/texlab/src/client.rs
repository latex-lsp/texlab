use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

use anyhow::{bail, Result};
use crossbeam_channel::Sender;
use dashmap::DashMap;
use lsp_server::{ErrorCode, Message, Request, RequestId, Response};
use lsp_types::{notification::ShowMessage, MessageType, ShowMessageParams};
use serde::{de::DeserializeOwned, Serialize};

use crate::server::options::Options;

#[derive(Debug)]
struct RawClient {
    sender: Sender<Message>,
    next_id: AtomicI32,
    pending: DashMap<RequestId, Sender<Response>>,
}

#[derive(Debug, Clone)]
pub struct LspClient {
    raw: Arc<RawClient>,
}

impl LspClient {
    pub fn new(sender: Sender<Message>) -> Self {
        let raw = Arc::new(RawClient {
            sender,
            next_id: AtomicI32::new(1),
            pending: DashMap::default(),
        });

        Self { raw }
    }

    pub fn send_notification<N>(&self, params: N::Params) -> Result<()>
    where
        N: lsp_types::notification::Notification,
        N::Params: Serialize,
    {
        self.raw
            .sender
            .send(lsp_server::Notification::new(N::METHOD.to_string(), params).into())?;
        Ok(())
    }

    pub fn send_request<R>(&self, params: R::Params) -> Result<R::Result>
    where
        R: lsp_types::request::Request,
        R::Params: Serialize,
        R::Result: DeserializeOwned,
    {
        let id = RequestId::from(self.raw.next_id.fetch_add(1, Ordering::SeqCst));

        let (tx, rx) = crossbeam_channel::bounded(1);
        self.raw.pending.insert(id.clone(), tx);

        self.raw
            .sender
            .send(Request::new(id, R::METHOD.to_string(), params).into())?;

        let response = rx.recv()?;
        let result = match response.error {
            Some(error) => bail!(error.message),
            None => response.result.unwrap_or_default(),
        };

        Ok(serde_json::from_value(result)?)
    }

    pub fn send_response(&self, response: lsp_server::Response) -> Result<()> {
        self.raw.sender.send(response.into())?;
        Ok(())
    }

    pub fn send_error(&self, id: RequestId, code: ErrorCode, message: String) -> Result<()> {
        self.send_response(lsp_server::Response::new_err(id, code as i32, message))?;
        Ok(())
    }

    pub fn recv_response(&self, response: lsp_server::Response) -> Result<()> {
        let (_, tx) = self
            .raw
            .pending
            .remove(&response.id)
            .expect("response with known request id received");

        tx.send(response)?;
        Ok(())
    }

    pub fn parse_options(&self, value: serde_json::Value) -> Result<Options> {
        let options = match serde_json::from_value(value) {
            Ok(new_options) => new_options,
            Err(why) => {
                let message = format!(
                    "The texlab configuration is invalid; using the default settings instead.\nDetails: {why}"
                );
                let typ = MessageType::WARNING;
                self.send_notification::<ShowMessage>(ShowMessageParams { message, typ })?;
                None
            }
        };

        Ok(options.unwrap_or_default())
    }
}
