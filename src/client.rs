use futures::future::BoxFuture;
use jsonrpc::client::FutureResult;
use jsonrpc_derive::{jsonrpc_client, jsonrpc_method};
use lsp_types::{ConfigurationParams, ShowMessageParams};

#[jsonrpc_client(LatexLspClient)]
pub trait LspClient {
    #[jsonrpc_method("workspace/configuration", kind = "request")]
    fn configuration(&self, params: ConfigurationParams) -> FutureResult<'_, serde_json::Value>;

    #[jsonrpc_method("window/showMessage", kind = "notification")]
    fn show_message(&self, params: ShowMessageParams) -> BoxFuture<'_, ()>;
}
