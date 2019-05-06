mod codec;

use crate::server::LatexLspServer;
use codec::LspCodec;
use futures::compat::*;
use futures::executor::ThreadPool;
use futures::lock::Mutex;
use futures::prelude::*;
use futures::task::*;
use jsonrpc::handle_message;
use std::sync::Arc;
use tokio::codec::{FramedRead, FramedWrite};
use tokio::prelude::{AsyncRead, AsyncWrite};

pub async fn listen<I, O>(server: LatexLspServer, input: I, output: O, pool: ThreadPool)
where
    I: AsyncRead + Send + Sync + 'static,
    O: AsyncWrite + Send + Sync + 'static,
{
    let server = Arc::new(server);
    let mut reader = FramedRead::new(input, LspCodec).compat();
    let writer = Arc::new(Mutex::new(FramedWrite::new(output, LspCodec).sink_compat()));

    while let Some(content) = await!(reader.next()) {
        let mut pool = pool.clone();
        let message = content.expect("Invalid message format");

        let server = Arc::clone(&server);
        let writer = Arc::clone(&writer);
        pool.spawn(async move {
            if let Some(response) = handle_message!(message, server) {
                let response = serde_json::to_string(&response).unwrap();
                let mut writer = await!(writer.lock());
                await!(writer.send(response)).expect("Cannot write into output")
            }
        })
        .unwrap();
    }
}
