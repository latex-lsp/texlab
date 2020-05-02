use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, ArgMatches,
};
use futures::{channel::mpsc, prelude::*};
use jsonrpc::MessageHandler;
use log::LevelFilter;
use std::{env, error, fs::OpenOptions, sync::Arc};

use texlab::server::LatexLspServer;
use texlab_protocol::{LatexLspClient, LspCodec};
use texlab_tex::DynamicDistribution;
use tokio_util::codec::{FramedRead, FramedWrite};

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let opts = app_from_crate!()
        .author("")
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Increase message verbosity (-vvvv for max verbosity)"),
        )
        .arg(
            Arg::with_name("quiet")
                .long("quiet")
                .short("q")
                .help("No output printed to stderr"),
        )
        .arg(
            Arg::with_name("log_file")
                .long("log-file")
                .value_name("FILE")
                .help("Send the logs to the given file"),
        )
        .get_matches();

    setup_logger(opts);

    let mut stdin = FramedRead::new(tokio::io::stdin(), LspCodec);
    let (stdout_tx, mut stdout_rx) = mpsc::channel(0);

    let client = Arc::new(LatexLspClient::new(stdout_tx.clone()));
    let server = Arc::new(LatexLspServer::new(
        DynamicDistribution::detect().await,
        Arc::clone(&client),
        Arc::new(env::current_dir().expect("failed to get working directory")),
    ));
    let mut handler = MessageHandler {
        server,
        client,
        output: stdout_tx,
    };

    tokio::spawn(async move {
        let mut stdout = FramedWrite::new(tokio::io::stdout(), LspCodec);
        loop {
            let message = stdout_rx.next().await.unwrap();
            stdout.send(message).await.unwrap();
        }
    });

    while let Some(json) = stdin.next().await {
        handler.handle(&json.unwrap()).await;
    }

    Ok(())
}

fn setup_logger(opts: ArgMatches) {
    let verbosity_level = if !opts.is_present("quiet") {
        match opts.occurrences_of("verbosity") {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    } else {
        LevelFilter::Off
    };

    let logger = fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("{} - {}", record.level(), message)))
        .level(verbosity_level)
        .filter(|metadata| metadata.target() == "jsonrpc" || metadata.target().contains("texlab"))
        .chain(std::io::stderr());

    let logger = match opts.value_of("log_file") {
        Some(log_file) => logger.chain(
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(log_file)
                .expect("failed to open log file"),
        ),
        None => logger,
    };

    logger.apply().expect("failed to initialize logger");
}
