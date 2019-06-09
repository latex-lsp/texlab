#![feature(async_await)]

use clap::*;
use futures::compat::*;
use futures::executor::*;
use futures::lock::Mutex;
use futures::prelude::*;
use jsonrpc::MessageHandler;
use std::sync::Arc;
use stderrlog::{ColorChoice, Timestamp};
use texlab::client::LatexLspClient;
use texlab::codec::LspCodec;
use texlab::server::LatexLspServer;
use tokio::codec::FramedRead;
use tokio_codec::FramedWrite;
use tokio_stdin_stdout;

fn main() {
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

    let mut pool = ThreadPool::new().expect("Failed to create the thread pool");
    let task = run(pool.clone());
    pool.run(task.unit_error()).unwrap();
}

async fn run(pool: ThreadPool) {
    let stdin = tokio_stdin_stdout::stdin(0);
    let stdout = tokio_stdin_stdout::stdout(0);
    let input = FramedRead::new(stdin, LspCodec).compat();
    let output = Arc::new(Mutex::new(FramedWrite::new(stdout, LspCodec).sink_compat()));
    let client = Arc::new(LatexLspClient::new(Arc::clone(&output)));
    let server = Arc::new(LatexLspServer::new(Arc::clone(&client)));
    let mut handler = MessageHandler {
        server,
        client,
        input,
        output,
        pool,
    };

    handler.listen().await;
}
