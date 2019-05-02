#![feature(await_macro, async_await, futures_api)]

mod build;
mod completion;
mod definition;
mod feature;
mod folding;
mod formatting;
mod highlight;
mod hover;
mod link;
mod lsp;
mod metadata;
mod range;
mod reference;
mod rename;
mod server;
mod syntax;
mod workspace;

use crate::completion::latex::data::actor::LatexComponentDatabaseActor;
use crate::server::LatexLspServer;
use crate::workspace::WorkspaceActor;
use clap::*;
use futures::executor::*;
use futures::prelude::*;
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

    let mut pool = ThreadPool::new().expect("Failed to create the thread pool");
    let task = run(pool.clone());
    pool.run(task.unit_error()).unwrap();
}

async fn run(pool: ThreadPool) {
    let workspace = await!(WorkspaceActor::spawn(pool.clone()));
    let server = LatexLspServer::new(workspace);
    let stdin = tokio_stdin_stdout::stdin(0);
    let stdout = tokio_stdin_stdout::stdout(0);

    await!(lsp::listen(server, stdin, stdout, pool.clone()));
}
