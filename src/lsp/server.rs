use crate::lsp::codec::LspCodec;
use futures::compat::*;
use futures::prelude::*;
use jsonrpc_core::{BoxFuture, Compatibility, Error, ErrorCode, IoHandler, Params, Value};
use lsp_types::*;
use serde_json::json;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;
use tokio::codec::{FramedRead, FramedWrite};
use tokio::io::{AsyncRead, AsyncWrite};

pub type LspFuture<T> =
    Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send + 'static>>;

pub trait LspServer {
    fn initialize(&self, params: InitializeParams) -> LspFuture<InitializeResult>;

    fn initialized(&self, params: InitializedParams);

    fn shutdown(&self, params: ()) -> LspFuture<()>;

    fn exit(&self, params: ());

    fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams);

    fn did_open(&self, params: DidOpenTextDocumentParams);

    fn did_change(&self, params: DidChangeTextDocumentParams);

    fn did_save(&self, params: DidSaveTextDocumentParams);

    fn did_close(&self, params: DidCloseTextDocumentParams);

    fn completion(&self, params: CompletionParams) -> LspFuture<CompletionList>;

    fn completion_resolve(&self, item: CompletionItem) -> LspFuture<CompletionItem>;

    fn hover(&self, params: TextDocumentPositionParams) -> LspFuture<Option<Hover>>;

    fn definition(&self, params: TextDocumentPositionParams) -> LspFuture<Vec<Location>>;

    fn references(&self, params: ReferenceParams) -> LspFuture<Vec<Location>>;

    fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> LspFuture<Vec<DocumentHighlight>>;

    fn document_symbol(&self, params: DocumentSymbolParams) -> LspFuture<Vec<DocumentSymbol>>;

    fn document_link(&self, params: DocumentLinkParams) -> LspFuture<Vec<DocumentLink>>;

    fn formatting(&self, params: DocumentFormattingParams) -> LspFuture<Vec<TextEdit>>;

    fn rename(&self, params: RenameParams) -> LspFuture<Option<WorkspaceEdit>>;

    fn folding_range(&self, params: FoldingRangeParams) -> LspFuture<Vec<FoldingRange>>;
}

pub fn build_io_handler<T>(server: T) -> IoHandler
where
    T: LspServer + Send + Sync + 'static,
{
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
                                message,
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
                    handler.add_notification($name, move |json: Params| {
                        match json.parse() {
                            Ok(params) => $request(&*server, params),
                            Err(error) => panic!(error),
                        }
                    });
                }
            )*;
        };
    }

    add_requests!(
        "initialize" -> LspServer::initialize,
        "shutdown" -> LspServer::shutdown,
        "textDocument/completion" -> LspServer::completion,
        "completionItem/resolve" -> LspServer::completion_resolve,
        "textDocument/hover" -> LspServer::hover,
        "textDocument/definition" -> LspServer::definition,
        "textDocument/references" -> LspServer::references,
        "textDocument/documentHighlight" -> LspServer::document_highlight,
        "textDocument/documentSymbol" -> LspServer::document_symbol,
        "textDocument/documentLink" -> LspServer::document_link,
        "textDocument/formatting" -> LspServer::formatting,
        "textDocument/rename" -> LspServer::rename,
        "textDocument/foldingRange" -> LspServer::folding_range
    );

    add_notifications!(
        "initialized" -> LspServer::initialized,
        "exit" -> LspServer::exit,
        "workspace/didChangeWatchedFiles" -> LspServer::did_change_watched_files,
        "textDocument/didOpen" -> LspServer::did_open,
        "textDocument/didChange" -> LspServer::did_change,
        "textDocument/didSave" -> LspServer::did_save,
        "textDocument/didClose" -> LspServer::did_close
    );

    handler
}
