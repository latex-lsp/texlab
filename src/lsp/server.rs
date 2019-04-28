use crate::server::LatexLspServer;
use futures::executor::ThreadPool;
use futures::prelude::*;
use futures::task::*;
use jsonrpc_core::{BoxFuture, Compatibility, Error, ErrorCode, IoHandler, Params, Value};
use serde_json::json;
use std::sync::Arc;

pub fn build_io_handler(server: LatexLspServer, pool: ThreadPool) -> IoHandler {
    let server = Arc::new(server);
    let mut handler = IoHandler::with_compatibility(Compatibility::V2);

    macro_rules! add_requests {
        ($($name:literal -> $request:path), *) => {
            $(
                {
                    let server = Arc::clone(&server);
                    handler.add_method($name, move |json: Params| -> BoxFuture<Value> {
                        let server = Arc::clone(&server);
                        let future = async move {
                            let params = json.parse()?;
                            let result = await!($request(&*server, params)).map_err(|message| Error {
                                code: ErrorCode::ServerError(-32000),
                                message: message.to_owned(),
                                data: None,
                            })?;

                            Ok(json!(result))
                        };

                        Box::new(future.boxed().compat())
                    });
                }
            )*;
        };
    }

    macro_rules! add_notifications {
        ($($name:literal -> $request:path), *) => {
            $(
                {
                    let server = Arc::clone(&server);
                    let pool = pool.clone();

                    handler.add_notification($name, move |json: Params| {
                        let server = Arc::clone(&server);
                        let mut pool = pool.clone();

                        let task = async move {
                            match json.parse() {
                                Ok(params) => await!($request(&*server, params)),
                                Err(error) => panic!(error),
                            }
                        };

                        pool.spawn(task).unwrap();
                    });
                }
            )*;
        };
    }

    add_requests!(
        "initialize" -> LatexLspServer::initialize,
        "shutdown" -> LatexLspServer::shutdown,
        "textDocument/completion" -> LatexLspServer::completion,
        "completionItem/resolve" -> LatexLspServer::completion_resolve,
        "textDocument/hover" -> LatexLspServer::hover,
        "textDocument/definition" -> LatexLspServer::definition,
        "textDocument/references" -> LatexLspServer::references,
        "textDocument/documentHighlight" -> LatexLspServer::document_highlight,
        "textDocument/documentSymbol" -> LatexLspServer::document_symbol,
        "textDocument/documentLink" -> LatexLspServer::document_link,
        "textDocument/formatting" -> LatexLspServer::formatting,
        "textDocument/rename" -> LatexLspServer::rename,
        "textDocument/foldingRange" -> LatexLspServer::folding_range
    );

    add_notifications!(
        "initialized" -> LatexLspServer::initialized,
        "exit" -> LatexLspServer::exit,
        "workspace/didChangeWatchedFiles" -> LatexLspServer::did_change_watched_files,
        "textDocument/didOpen" -> LatexLspServer::did_open,
        "textDocument/didChange" -> LatexLspServer::did_change,
        "textDocument/didSave" -> LatexLspServer::did_save,
        "textDocument/didClose" -> LatexLspServer::did_close
    );

    handler
}
