use futures::future::Future;
use jsonrpc_core::*;
use lsp_types::*;
use serde_json::json;
use std::rc::Rc;
use std::sync::Arc;

pub type LspResult<T> = Box<Future<Item = T, Error = ()> + Send>;

pub trait LspServer {
    fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult>;

    fn initialized(&self, params: InitializedParams);

    fn shutdown(&self, params: ()) -> LspResult<()>;

    fn exit(&self, params: ());

    fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams);

    fn did_open(&self, params: DidOpenTextDocumentParams);

    fn did_change(&self, params: DidChangeTextDocumentParams);

    fn did_save(&self, params: DidSaveTextDocumentParams);

    fn did_close(&self, params: DidCloseTextDocumentParams);

    fn completion(&self, params: CompletionParams) -> LspResult<CompletionList>;

    fn completion_resolve(&self, item: CompletionItem) -> LspResult<CompletionItem>;

    fn hover(&self, params: TextDocumentPositionParams) -> LspResult<Option<Hover>>;

    fn definition(&self, params: TextDocumentPositionParams) -> LspResult<Vec<Location>>;

    fn references(&self, params: ReferenceParams) -> LspResult<Vec<Location>>;

    fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> LspResult<Vec<DocumentHighlight>>;

    fn document_symbol(&self, params: DocumentSymbolParams) -> LspResult<Vec<DocumentSymbol>>;

    fn document_link(&self, params: DocumentLinkParams) -> LspResult<Vec<DocumentLink>>;

    fn formatting(&self, params: DocumentFormattingParams) -> LspResult<Vec<TextEdit>>;

    fn rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>>;

    fn folding_range(&self, params: FoldingRangeParams) -> LspResult<Vec<FoldingRange>>;
}

pub fn build_io_handler<T>(server: Arc<T>) -> IoHandler
where
    T: LspServer + Send + Sync + 'static,
{
    let mut handler = IoHandler::with_compatibility(Compatibility::V2);

    macro_rules! add_requests {
        ($($name:literal -> $request:path), *) => {
            $(
                {
                    let server = Arc::clone(&server);
                    handler.add_method($name, move |json: Params| -> BoxFuture<Value> {
                        match json.parse() {
                            Ok(params) => Box::new(
                                $request(&*server, params)
                                    .map(|res| json!(res))
                                    .map_err(|_| Error::new(ErrorCode::InternalError)),
                            ),
                            Err(error) => Box::new(futures::failed(error)),
                        }
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
