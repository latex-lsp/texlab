mod range;
mod syntax;
mod lsp;
mod server;

use lsp::server::ServerBuilder;
use server::LatexLspServer;
use tokio;
use tokio_stdin_stdout;

fn main() {
    let server = LatexLspServer;
    let builder = ServerBuilder::new(server);

    let stdin = tokio_stdin_stdout::stdin(0).make_sendable();
    let stdout = tokio_stdin_stdout::stdout(0).make_sendable();
    tokio::run(builder.listen(stdin, stdout));
}
