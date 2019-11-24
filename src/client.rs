use futures_boxed::boxed;
use jsonrpc::client::Result;
use jsonrpc_derive::{jsonrpc_client, jsonrpc_method};
use lsp_types::*;

#[jsonrpc_client(LatexLspClient)]
pub trait LspClient {
    #[jsonrpc_method("workspace/configuration", kind = "request")]
    #[boxed]
    async fn configuration(&self, params: ConfigurationParams) -> Result<serde_json::Value>;

    #[jsonrpc_method("window/showMessage", kind = "notification")]
    #[boxed]
    async fn show_message(&self, params: ShowMessageParams);

    #[jsonrpc_method("client/registerCapability", kind = "request")]
    #[boxed]
    async fn register_capability(&self, params: RegistrationParams) -> Result<()>;

    #[jsonrpc_method("textDocument/publishDiagnostics", kind = "notification")]
    #[boxed]
    async fn publish_diagnostics(&self, params: PublishDiagnosticsParams);

    #[jsonrpc_method("$/progress", kind = "notification")]
    #[boxed]
    async fn progress(&self, params: ProgressParams);

    #[jsonrpc_method("window/workDoneProgress/create", kind = "request")]
    #[boxed]
    async fn work_done_progress_create(&self, params: WorkDoneProgressCreateParams) -> Result<()>;

    #[jsonrpc_method("window/logMessage", kind = "notification")]
    #[boxed]
    async fn log_message(&self, params: LogMessageParams);
}
