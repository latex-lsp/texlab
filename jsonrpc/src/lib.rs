#![feature(await_macro, async_await)]

pub mod client;
pub mod server;
mod types;

pub use self::{
    client::{Client, ResponseHandler},
    server::{handle_notification, handle_request, Server, EventHandler},
    types::*,
};

use futures::executor::ThreadPool;
use futures::lock::Mutex;
use futures::prelude::*;
use futures::task::*;
use std::sync::Arc;

pub struct MessageHandler<S, H, I, O> {
    server: Arc<S>,
    response_handler: Arc<H>,
    input: I,
    output: Arc<Mutex<O>>,
    pool: ThreadPool,
}

impl<S, H, I, O> MessageHandler<S, H, I, O>
where
    S: Server + EventHandler + Send + Sync + 'static,
    H: ResponseHandler + Send + Sync + 'static,
    I: Stream<Item = std::io::Result<String>> + Unpin,
    O: Sink<String> + Unpin + Send + 'static,
{
    pub fn new(
        server: S,
        response_handler: Arc<H>,
        input: I,
        output: Arc<Mutex<O>>,
        pool: ThreadPool,
    ) -> Self {
        MessageHandler {
            server: Arc::new(server),
            response_handler,
            input,
            output,
            pool,
        }
    }

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
                        await!(server.handle_events());
                    };

                    self.pool.spawn(handler).unwrap();
                }
                Ok(Message::Notification(notification)) => {
                    self.server.handle_notification(notification);

                    let server = Arc::clone(&self.server);
                    self.pool.spawn(async move {
                       await!(server.handle_events());
                    });
                }
                Ok(Message::Response(response)) => {
                    await!(self.response_handler.handle(response));
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
