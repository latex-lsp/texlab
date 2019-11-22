use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};
use futures::channel::mpsc;
use futures::prelude::*;
use jsonrpc::MessageHandler;
use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use stderrlog::{ColorChoice, Timestamp};
use texlab::client::LatexLspClient;
use texlab::codec::LspCodec;
use texlab::server::LatexLspServer;
use tokio::codec::{FramedRead, FramedWrite};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    texlab::citeproc::JavaScriptEngine::initialize();
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
        .module("jsonrpc")
        .verbosity(matches.occurrences_of("verbosity") as usize)
        .quiet(matches.is_present("quiet"))
        .timestamp(Timestamp::Off)
        .color(ColorChoice::Never)
        .init()
        .unwrap();

    let stdin = FramedRead::new(tokio::io::stdin(), LspCodec);
    let (stdout_tx, mut stdout_rx) = mpsc::channel(0);

    let client = Arc::new(LatexLspClient::new(stdout_tx.clone()));
    let server = Arc::new(LatexLspServer::new(
        Arc::new(tex::Distribution::detect().await),
        Arc::clone(&client),
    ));
    let mut handler = MessageHandler {
        server,
        client,
        input: stdin,
        output: stdout_tx,
    };

    tokio::spawn(async move {
        let mut stdout = FramedWrite::new(tokio::io::stdout(), LspCodec);
        loop {
            let message = stdout_rx.next().await.unwrap();
            stdout.send(message).await.unwrap();
            std::io::stdout().flush().unwrap(); // Workaround for tokio-rs/tokio#1527
        }
    });

    handler.listen().await;
    Ok(())
}
