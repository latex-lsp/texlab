use std::sync::Mutex;

use anyhow::{anyhow, Result};
use crossbeam_channel::{Receiver, Sender};
use lsp_server::{Message, ResponseError};
use serde::{de::DeserializeOwned, Serialize};

use crate::req_queue::{OutgoingData, ReqQueue};

pub fn send_notification<N>(lsp_sender: &Sender<Message>, params: N::Params) -> Result<()>
where
    N: lsp_types::notification::Notification,
    N::Params: Serialize,
{
    lsp_sender.send(lsp_server::Notification::new(N::METHOD.to_string(), params).into())?;
    Ok(())
}

pub fn send_request<R>(
    req_queue: &Mutex<ReqQueue>,
    lsp_sender: &Sender<Message>,
    params: R::Params,
) -> Result<R::Result>
where
    R: lsp_types::request::Request,
    R::Params: Serialize,
    R::Result: DeserializeOwned,
{
    let receiver = register_outgoing_request::<R>(req_queue, lsp_sender, params)?;
    let params = receiver.recv()?.map_err(|err| anyhow!(err.message))?;
    let result = serde_json::from_value(params)?;
    Ok(result)
}

fn register_outgoing_request<R>(
    req_queue: &Mutex<ReqQueue>,
    lsp_sender: &Sender<Message>,
    params: R::Params,
) -> Result<Receiver<Result<serde_json::Value, ResponseError>>>
where
    R: lsp_types::request::Request,
    R::Params: Serialize,
    R::Result: DeserializeOwned,
{
    let mut req_queue = req_queue.lock().unwrap();
    let (sender, receiver) = crossbeam_channel::bounded(1);
    let method = R::METHOD.to_string();
    let data = OutgoingData { sender };
    let req = req_queue.outgoing.register(method, params, data);
    drop(req_queue);
    lsp_sender.send(req.into())?;
    Ok(receiver)
}
