mod codec;
mod server;

use crate::server::LatexLspServer;
use codec::LspCodec;
use futures::compat::*;
use futures::prelude::*;
use server::build_io_handler;
use tokio::codec::{FramedRead, FramedWrite};
use tokio::prelude::{AsyncRead, AsyncWrite};

pub async fn listen<I, O>(server: LatexLspServer, input: I, output: O) -> Result<(), ()>
where
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
