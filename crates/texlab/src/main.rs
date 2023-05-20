use std::{fs::OpenOptions, io, path::PathBuf};

use anyhow::Result;
use clap::{ArgAction, Parser};
use log::LevelFilter;
use lsp_server::Connection;
use texlab::Server;

/// An implementation of the Language Server Protocol for LaTeX
#[derive(Debug, Parser)]
#[clap(version)]
struct Opts {
    /// Increase message verbosity (-vvvv for max verbosity)
    #[clap(short, long, action = ArgAction::Count)]
    verbosity: u8,

    /// No output printed to stderr
    #[clap(short, long)]
    quiet: bool,

    /// Write the logging output to FILE
    #[clap(long, name = "FILE", value_parser)]
    log_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    setup_logger(opts);

    let (connection, threads) = Connection::stdio();
    Server::new(connection).run()?;
    threads.join()?;

    Ok(())
}

fn setup_logger(opts: Opts) {
    let verbosity_level = if !opts.quiet {
        match opts.verbosity {
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
        .chain(io::stderr());

    let logger = match opts.log_file {
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
