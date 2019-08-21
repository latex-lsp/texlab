use clap::*;
use futures::channel::mpsc;
use futures::compat::*;
use futures::future;
use futures::prelude::*;
use jsonrpc::MessageHandler;
use std::sync::Arc;
use stderrlog::{ColorChoice, Timestamp};
use texlab::client::LatexLspClient;
use texlab::codec::LspCodec;
use texlab::server::LatexLspServer;
use tokio::codec::FramedRead;
use tokio_codec::FramedWrite;

#[runtime::main(runtime_tokio::Tokio)]
async fn main() {
    // Force initialization of the underlying JavaScript engine
    // to decrease the time of the initial citation rendering
    drop(texlab::citeproc::render_citation(""));

    let matches = app_from_crate!()
        .author("")
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Increase message verbosity"),
        )
        .arg(
            Arg::with_name("quiet")
                .long("quiet")
                .short("q")
                .help("No output printed to stderr"),
        )
        .get_matches();

    stderrlog::new()
        .module(module_path!())
        .verbosity(matches.occurrences_of("verbosity") as usize)
        .quiet(matches.is_present("quiet"))
        .timestamp(Timestamp::Off)
        .color(ColorChoice::Never)
        .init()
        .unwrap();

    let stdin = FramedRead::new(tokio::io::stdin(), LspCodec).compat();
    let stdout = FramedWrite::new(tokio::io::stdout(), LspCodec).sink_compat();
    let (stdout_tx, stdout_rx) = mpsc::channel(0);

    let client = Arc::new(LatexLspClient::new(stdout_tx.clone()));
    let server = Arc::new(LatexLspServer::new(Arc::clone(&client)));
    let mut handler = MessageHandler {
        server,
        client,
        input: stdin,
        output: stdout_tx,
    };

    let stdout_handle = runtime::spawn(async move {
        stdout_rx.map(|x| Ok(x)).forward(stdout).await.unwrap();
    });

    future::join(handler.listen(), stdout_handle).await;
}
