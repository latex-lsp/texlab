use crate::types::*;
use futures::channel::oneshot;
use futures::future::BoxFuture;
use futures::lock::Mutex;
use futures::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, Error>;

pub type FutureResult<'a, T> = BoxFuture<'a, Result<T>>;

pub trait ResponseHandler {
    fn handle(&self, response: Response) -> BoxFuture<'_, ()>;
}

pub struct Client<O> {
    output: Arc<Mutex<O>>,
    request_id: AtomicI32,
    queue: Mutex<HashMap<Id, oneshot::Sender<Result<serde_json::Value>>>>,
}

impl<O> Client<O>
where
    O: Output,
{
    pub fn new(output: Arc<Mutex<O>>) -> Self {
        Client {
            output,
            request_id: AtomicI32::new(0),
            queue: Mutex::new(HashMap::new()),
        }
    }

    pub async fn send_request<T: Serialize>(
        &self,
        method: String,
        params: T,
    ) -> Result<serde_json::Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = Request::new(method, json!(params), id);

        let (sender, receiver) = oneshot::channel();
        {
            let mut queue = self.queue.lock().await;
            queue.insert(request.id, sender);
        }

        self.send(Message::Request(request)).await;
        receiver.await.unwrap()
    }

    pub async fn send_notification<T: Serialize>(&self, method: String, params: T) {
        let notification = Notification::new(method, json!(params));
        self.send(Message::Notification(notification)).await;
    }

    async fn send(&self, message: Message) {
        let json = serde_json::to_string(&message).unwrap();
        let mut output = self.output.lock().await;
        output.send(json).await.unwrap();
    }
}

impl<O> ResponseHandler for Client<O>
where
    O: Output,
{
    fn handle(&self, response: Response) -> BoxFuture<'_, ()> {
        let task = async move {
            let id = response.id.expect("Expected response with id");
            let mut queue = self.queue.lock().await;
            let sender = queue.remove(&id).expect("Unexpected response received");

            let result = match response.error {
                Some(why) => Err(why),
                None => Ok(response.result.unwrap_or(serde_json::Value::Null)),
            };
            sender.send(result).unwrap();
        };

        task.boxed()
    }
}
