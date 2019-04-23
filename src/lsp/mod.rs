mod codec;
mod server;

use crate::server::LatexLspServer;
use codec::LspCodec;
use futures::compat::*;
use futures::executor::ThreadPool;
use futures::lock::Mutex;
use futures::prelude::*;
use futures::task::*;
use server::build_io_handler;
use std::sync::Arc;
use tokio::codec::{FramedRead, FramedWrite};
use tokio::prelude::{AsyncRead, AsyncWrite};

pub async fn listen<I, O>(server: LatexLspServer, input: I, output: O, pool: ThreadPool)
where
    I: AsyncRead + Send + Sync + 'static,
    O: AsyncWrite + Send + Sync + 'static,
{
    let handler = Arc::new(Mutex::new(build_io_handler(server)));
    let mut reader = FramedRead::new(input, LspCodec).compat();
    let writer = Arc::new(Mutex::new(FramedWrite::new(output, LspCodec).sink_compat()));

    while let Some(content) = await!(reader.next()) {
        let mut pool = pool.clone();
        let handler = Arc::clone(&handler);
        let writer = Arc::clone(&writer);

        pool.spawn(async move {
            let message = content.expect("Invalid message format");
            let handler = await!(handler.lock());

            if let Ok(Some(response)) = await!(handler.handle_request(&message).compat()) {
                let mut writer = await!(writer.lock());
                await!(writer.send(response)).expect("Cannot write into output")
            }
        })
        .unwrap();
    }
}
