pub mod client;
pub mod server;
mod types;

pub use self::{
    client::{Client, ResponseHandler},
    server::{handle_notification, handle_request, ActionHandler, RequestHandler},
    types::*,
};

use futures::channel::*;
use futures::prelude::*;
use std::io;
use std::sync::Arc;

pub struct MessageHandler<S, C, I> {
    pub server: Arc<S>,
    pub client: Arc<C>,
    pub input: I,
    pub output: mpsc::Sender<String>,
}

impl<S, C, I> MessageHandler<S, C, I>
where
    S: RequestHandler + ActionHandler + Send + Sync + 'static,
    C: ResponseHandler + Send + Sync + 'static,
    I: Stream<Item = io::Result<String>> + Unpin,
{
    pub async fn listen(&mut self) {
        while let Some(json) = self.input.next().await {
            let message = serde_json::from_str(&json.unwrap()).map_err(|_| Error::parse_error());
            match message {
                Ok(Message::Request(request)) => {
                    let server = Arc::clone(&self.server);
                    let mut output = self.output.clone();

                    drop(runtime::spawn(async move {
                        let response = server.handle_request(request).await;
                        let json = serde_json::to_string(&response).unwrap();
                        output.send(json).await.unwrap();
                        server.execute_actions().await;
                    }));
                }
                Ok(Message::Notification(notification)) => {
                    self.server.handle_notification(notification);
                    let server = Arc::clone(&self.server);
                    drop(runtime::spawn(async move {
                        server.execute_actions().await;
                    }));
                }
                Ok(Message::Response(response)) => {
                    self.client.handle(response).await;
                }
                Err(why) => {
                    let response = Response::error(why, None);
                    let json = serde_json::to_string(&response).unwrap();
                    self.output.send(json).await.unwrap();
                }
            }
        }
    }
}
