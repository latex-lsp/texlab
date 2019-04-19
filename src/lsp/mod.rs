mod codec;
mod server;

pub use server::{LspFuture, LspServer};

use codec::LspCodec;
use futures::compat::*;
use futures::prelude::*;
use jsonrpc_core::IoHandler;
use server::build_io_handler;
use tokio::codec::{FramedRead, FramedWrite};
use tokio::prelude::{AsyncRead, AsyncWrite};

pub async fn listen<T, I, O>(server: T, input: I, output: O) -> Result<(), ()>
where
    T: LspServer + Send + Sync + 'static,
    I: AsyncRead,
    O: AsyncWrite,
{
    let handler = build_io_handler(server);
    let mut reader = FramedRead::new(input, LspCodec).compat();
    let mut writer = FramedWrite::new(output, LspCodec).sink_compat();

    while let Some(content) = await!(reader.next()) {
        let message = content.expect("Invalid message format");
        if let Some(response) = await!(handler.handle_request(&message).compat())? {
            await!(writer.send(response)).expect("Cannot write into output")
        }
    }

    Ok(())
}
