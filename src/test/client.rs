use crate::{jsonrpc::client::Result, protocol::*};
use futures_boxed::boxed;
use jsonrpc_derive::{jsonrpc_client, jsonrpc_method};

#[jsonrpc_client(TestLatexLspClient)]
pub trait TestLspClient {
    #[jsonrpc_method("initialize", kind = "request")]
    #[boxed]
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult>;

    #[jsonrpc_method("initialized", kind = "notification")]
    #[boxed]
    async fn initialized(&self, params: InitializedParams);

    #[jsonrpc_method("shutdown", kind = "request")]
    #[boxed]
    async fn shutdown(&self, params: ()) -> Result<()>;

    #[jsonrpc_method("exit", kind = "notification")]
    #[boxed]
    async fn exit(&self, params: ());

    #[jsonrpc_method("textDocument/didOpen", kind = "notification")]
    #[boxed]
    async fn did_open(&self, params: DidOpenTextDocumentParams);

    #[jsonrpc_method("textDocument/didChange", kind = "notification")]
    #[boxed]
    async fn did_change(&self, params: DidChangeTextDocumentParams);

    #[jsonrpc_method("workspace/didChangeConfiguration", kind = "notification")]
    #[boxed]
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams);

    #[jsonrpc_method("textDocument/documentLink", kind = "request")]
    #[boxed]
    async fn document_link(&self, params: DocumentLinkParams) -> Result<Vec<DocumentLink>>;
}
