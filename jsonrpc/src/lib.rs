#![feature(await_macro, async_await)]

mod server;
mod types;

pub use self::server::*;
pub use self::types::*;

use futures::executor::ThreadPool;
use futures::lock::Mutex;
use futures::prelude::*;
use futures::task::*;
use std::sync::Arc;

pub struct MessageHandler<S, I, O> {
    server: Arc<S>,
    input: I,
    output: Arc<Mutex<O>>,
    pool: ThreadPool,
}

impl<S, I, O> MessageHandler<S, I, O>
where
    S: Server + Send + Sync + 'static,
    I: Stream<Item = std::io::Result<String>> + Unpin,
    O: Sink<String> + Unpin + Send + 'static,
{
    pub fn new(server: S, input: I, output: O, pool: ThreadPool) -> Self {
        MessageHandler {
            server: Arc::new(server),
            input,
            output: Arc::new(Mutex::new(output)),
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
                    };

                    self.pool.spawn(handler).unwrap();
                }
                Ok(Message::Notification(notification)) => {
                    self.server.handle_notification(notification);
                }
                Ok(Message::Response(response)) => unimplemented!("{:?}", response),
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
