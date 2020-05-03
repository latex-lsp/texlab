use super::types::*;
use async_trait::async_trait;
use chashmap::CHashMap;
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use serde::Serialize;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait ResponseHandler {
    async fn handle(&self, response: Response);
}

#[derive(Debug)]
pub struct Client {
    output: mpsc::Sender<String>,
    request_id: AtomicU64,
    senders_by_id: CHashMap<Id, oneshot::Sender<Result<serde_json::Value>>>,
}

impl Client {
    pub fn new(output: mpsc::Sender<String>) -> Self {
        Self {
            output,
            request_id: AtomicU64::new(0),
            senders_by_id: CHashMap::new(),
        }
    }

    pub async fn send_request<T: Serialize>(
        &self,
        method: String,
        params: T,
    ) -> Result<serde_json::Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = Request::new(method, json!(params), Id::Number(id));

        let (result_tx, result_rx) = oneshot::channel();
        self.senders_by_id.insert(request.id.clone(), result_tx);
        self.send(Message::Request(request)).await;

        result_rx.await.unwrap()
    }

    pub async fn send_notification<T: Serialize>(&self, method: String, params: T) {
        let notification = Notification::new(method, json!(params));
        self.send(Message::Notification(notification)).await;
    }

    async fn send(&self, message: Message) {
        let mut output = self.output.clone();
        let json = serde_json::to_string(&message).unwrap();
        output.send(json).await.unwrap();
    }
}

#[async_trait]
impl ResponseHandler for Client {
    async fn handle(&self, response: Response) {
        let id = response.id.expect("Expected response with id");
        let result = match response.error {
            Some(why) => Err(why),
            None => Ok(response.result.unwrap_or(serde_json::Value::Null)),
        };

        let result_tx = self
            .senders_by_id
            .remove(&id)
            .expect("Unexpected response received");
        result_tx.send(result).unwrap();
    }
}
