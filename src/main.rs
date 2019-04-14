mod lsp;
mod range;
mod server;
mod syntax;

use lsp::server::ServerBuilder;
use server::LatexLspServer;
use stderrlog::*;
use tokio;
use tokio_stdin_stdout;

fn main() {
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Off)
        .init()
        .unwrap();

    let server = LatexLspServer;
    let builder = ServerBuilder::new(server);

    let stdin = tokio_stdin_stdout::stdin(0).make_sendable();
    let stdout = tokio_stdin_stdout::stdout(0).make_sendable();
    tokio::run(builder.listen(stdin, stdout));
}
