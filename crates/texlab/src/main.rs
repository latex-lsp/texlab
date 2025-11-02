use std::{fs::OpenOptions, io, path::PathBuf};

use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};
use log::LevelFilter;
use lsp_server::Connection;
use texlab::Server;
use url::Url;

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

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Runs the language server in a editor context using STDIN and STDOUT.
    Run,

    /// Opens a document at a specific line.
    ///
    /// This command can be used to implement inverse search in an editor-agnostic way.
    InverseSearch(InverseSearchOpts),
}

/// Options for the inverse search subcommand.
#[derive(Debug, Parser)]
struct InverseSearchOpts {
    /// The path to the document to open.
    #[clap(short, long, name = "FILE", value_parser)]
    input: PathBuf,

    /// The zero-based line number of the document to jump to (alias for --line0).
    #[clap(short, long)]
    line: Option<u32>,

    /// The zero-based line number of the document to jump to.
    #[clap(long)]
    line0: Option<u32>,

    /// The one-based line number of the document to jump to.
    #[clap(long)]
    line1: Option<u32>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    setup_logger(&opts);

    match opts.command.unwrap_or(Command::Run) {
        Command::Run => {
            let (connection, threads) = Connection::stdio();
            Server::exec(connection)?;
            threads.join()?;
        }
        Command::InverseSearch(opts) => {
            let Some(uri) = opts
                .input
                .canonicalize()
                .ok()
                .and_then(|path| Url::from_file_path(path).ok())
            else {
                eprintln!("Failed to convert input path to a URI.");
                std::process::exit(-1);
            };

            let Some(line) = opts
                .line
                .or_else(|| opts.line0)
                .or_else(|| opts.line1.and_then(|l| l.checked_sub(1)))
            else {
                if opts.line1.is_some() {
                    eprintln!("--line1 must be a positive integer.");
                } else {
                    eprintln!("Either --line, --line0 or --line1 must be specified.");
                }
                std::process::exit(-1);
            };

            let params = lsp_types::TextDocumentPositionParams::new(
                lsp_types::TextDocumentIdentifier::new(uri),
                lsp_types::Position::new(line, 0),
            );

            if let Err(why) = ipc::send_request(params) {
                eprintln!(
                    "Failed to send inverse search request to the main instance. Is the server running?"
                );
                eprintln!("Details: {why:?}");
                std::process::exit(-1);
            }
        }
    }

    Ok(())
}

fn setup_logger(opts: &Opts) {
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

    let logger = match &opts.log_file {
        Some(log_file) => logger.chain(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(log_file)
                .expect("failed to open log file"),
        ),
        None => logger,
    };

    logger.apply().expect("failed to initialize logger");
}
