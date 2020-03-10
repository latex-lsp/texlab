pub mod client;
pub mod server;
mod types;

pub use self::{
    client::{Client, ResponseHandler},
    server::{handle_notification, handle_request, Middleware, RequestHandler},
    types::*,
};

use futures::{channel::mpsc, prelude::*};
use log::error;
use std::sync::Arc;

#[derive(Debug)]
pub struct MessageHandler<S, C> {
    pub server: Arc<S>,
    pub client: Arc<C>,
    pub output: mpsc::Sender<String>,
}

impl<S, C> MessageHandler<S, C>
where
    S: RequestHandler + Middleware + Send + Sync + 'static,
    C: ResponseHandler + Send + Sync + 'static,
{
    pub async fn handle(&mut self, json: &str) {
        self.server.before_message().await;

        match serde_json::from_str(json).map_err(|_| Error::parse_error()) {
            Ok(Message::Request(request)) => {
                let server = Arc::clone(&self.server);
                let mut output = self.output.clone();
                tokio::spawn(async move {
                    let response = server.handle_request(request).await;
                    if let Some(error) = response.error.as_ref() {
                        error!("{:?}", error);
                    }
                    let json = serde_json::to_string(&response).unwrap();
                    output.send(json).await.unwrap();
                    server.after_message().await;
                });
            }
            Ok(Message::Notification(notification)) => {
                self.server.handle_notification(notification).await;
                self.after_message();
            }
            Ok(Message::Response(response)) => {
                self.client.handle(response).await;
                self.after_message();
            }
            Err(why) => {
                let response = Response::error(why, None);
                let json = serde_json::to_string(&response).unwrap();
                self.output.send(json).await.unwrap();
                self.after_message();
            }
        };
    }

    fn after_message(&self) {
        let server = Arc::clone(&self.server);
        tokio::spawn(async move {
            server.after_message().await;
        });
    }
}
