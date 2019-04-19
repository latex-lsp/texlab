#![feature(await_macro, async_await, futures_api)]

mod build;
mod formatting;
mod lsp;
mod range;
mod server;
mod syntax;

use clap::*;
use futures::prelude::*;
use server::LatexLspServer;
use std::sync::Arc;
use tokio;
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
        .timestamp(stderrlog::Timestamp::Off)
        .init()
        .unwrap();

    let future = async {
        let server = LatexLspServer;
        let stdin = tokio_stdin_stdout::stdin(0).make_sendable();
        let stdout = tokio_stdin_stdout::stdout(0).make_sendable();
        await!(lsp::listen(server, stdin, stdout))
    };

    tokio::run(future.boxed().compat());
}
