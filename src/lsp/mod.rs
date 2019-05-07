mod codec;

use crate::server::LatexLspServer;
use codec::LspCodec;
use futures::compat::*;
use futures::executor::ThreadPool;
use futures::lock::Mutex;
use futures::prelude::*;
use futures::task::*;
use jsonrpc::*;
use std::sync::Arc;
use tokio::codec::{FramedRead, FramedWrite};
use tokio::prelude::{AsyncRead, AsyncWrite};

pub async fn listen<I, O>(server: LatexLspServer, input: I, output: O, mut pool: ThreadPool)
where
    I: AsyncRead + Send + Sync + 'static,
    O: AsyncWrite + Send + Sync + 'static,
{
    let server = Arc::new(server);
    let mut reader = FramedRead::new(input, LspCodec).compat();
    let writer = Arc::new(Mutex::new(FramedWrite::new(output, LspCodec).sink_compat()));

    while let Some(content) = await!(reader.next()) {
        let message =
            serde_json::from_str(&content.expect("Invalid message format")).map_err(|_| Error {
                code: ErrorCode::ParseError,
                message: "Could not parse the input".to_owned(),
                data: serde_json::Value::Null,
            });

        match message {
            Ok(Message::Request(request)) => {
                let server = Arc::clone(&server);
                let writer = Arc::clone(&writer);
                let task = async move {
                    let response = await!(server.handle_request(request));
                    let mut writer = await!(writer.lock());
                    await!(writer.send(serde_json::to_string(&response).unwrap()))
                        .expect("Cannot write into output");
                };
                pool.spawn(task).unwrap();
            }
            Ok(Message::Notification(notification)) => {
                server.handle_notification(notification);
            }
            Ok(Message::Response(_)) => unimplemented!(),
            Err(why) => {
                let response = Response::new(serde_json::Value::Null, Some(why), None);
                let mut writer = await!(writer.lock());
                await!(writer.send(serde_json::to_string(&response).unwrap()))
                    .expect("Cannot write into output");
            }
        }
    }
}
