#![feature(async_await, trait_alias, async_closure)]

pub mod client;
pub mod server;
mod types;

pub use self::{
    client::{Client, ResponseHandler},
    server::{handle_notification, handle_request, ActionHandler, RequestHandler},
    types::*,
};

use futures::lock::Mutex;
use futures::prelude::*;
use std::sync::Arc;

pub struct MessageHandler<S, C, I, O> {
    pub server: Arc<S>,
    pub client: Arc<C>,
    pub input: I,
    pub output: Arc<Mutex<O>>,
}

impl<S, C, I, O> MessageHandler<S, C, I, O>
where
    S: RequestHandler + ActionHandler + Send + Sync + 'static,
    C: ResponseHandler + Send + Sync + 'static,
    I: Input,
    O: Output + 'static,
{
    pub async fn listen(&mut self) {
        while let Some(json) = self.input.next().await {
            let message = serde_json::from_str(&json.expect("")).map_err(|_| Error {
                code: ErrorCode::ParseError,
                message: "Could not parse the input".to_owned(),
                data: serde_json::Value::Null,
            });

            match message {
                Ok(Message::Request(request)) => {
                    let server = Arc::clone(&self.server);
                    let output = Arc::clone(&self.output);
                    runtime::spawn(async move {
                        let response = server.handle_request(request).await;
                        let json = serde_json::to_string(&response).unwrap();
                        {
                            let mut output = output.lock().await;
                            output.send(json).await.unwrap();
                        }
                        server.execute_actions().await;
                    });
                }
                Ok(Message::Notification(notification)) => {
                    self.server.handle_notification(notification);
                    let server = Arc::clone(&self.server);
                    runtime::spawn(async move {
                        server.execute_actions().await;
                    });
                }
                Ok(Message::Response(response)) => {
                    self.client.handle(response).await;
                }
                Err(why) => {
                    let response = Response::error(why, None);
                    let json = serde_json::to_string(&response).unwrap();
                    {
                        let mut output = self.output.lock().await;
                        output.send(json).await.unwrap();
                    }
                }
            }
        }
    }
}
