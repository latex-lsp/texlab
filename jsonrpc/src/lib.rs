#![feature(await_macro, async_await)]

pub mod client;
pub mod server;
mod types;

pub use self::{
    client::{Client, ResponseHandler},
    server::{handle_notification, handle_request, ActionHandler, RequestHandler},
    types::*,
};

use futures::executor::ThreadPool;
use futures::lock::Mutex;
use futures::prelude::*;
use futures::task::*;
use std::sync::Arc;

pub struct MessageHandler<S, C, I, O> {
    pub server: Arc<S>,
    pub client: Arc<C>,
    pub input: I,
    pub output: Arc<Mutex<O>>,
    pub pool: ThreadPool,
}

impl<S, C, I, O> MessageHandler<S, C, I, O>
    where
        S: RequestHandler + ActionHandler + Send + Sync + 'static,
        C: ResponseHandler + Send + Sync + 'static,
        I: Stream<Item = std::io::Result<String>> + Unpin,
        O: Sink<String> + Unpin + Send + 'static,
{
    pub async fn listen(&mut self) {
        while let Some(json) = await!(self.input.next()) {
            let message = serde_json::from_str(&json.expect("")).map_err(|_| Error {
                code: ErrorCode::ParseError,
                message: "Could not parse the input".to_owned(),
                data: serde_json::Value::Null,
            });

            match message {
                Ok(Message::Request(request)) => {
                    let server = Arc::clone(&self.server);
                    let output = Arc::clone(&self.output);
                    let handler = async move {
                        let response = await!(server.handle_request(request));
                        let json = serde_json::to_string(&response).unwrap();
                        let mut output = await!(output.lock());
                        await!(output.send(json));
                        await!(server.execute_actions());
                    };

                    self.pool.spawn(handler).unwrap();
                }
                Ok(Message::Notification(notification)) => {
                    self.server.handle_notification(notification);

                    let server = Arc::clone(&self.server);
                    let handler = async move {
                        await!(server.execute_actions());
                    };

                    self.pool.spawn(handler).unwrap();
                }
                Ok(Message::Response(response)) => {
                    await!(self.client.handle(response));
                }
                Err(why) => {
                    let response = Response::error(why, None);
                    let json = serde_json::to_string(&response).unwrap();
                    let mut output = await!(self.output.lock());
                    await!(output.send(json));
                }
            }
        }
    }
}