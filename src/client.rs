use crate::formatting::bibtex::BibtexFormattingOptions;
use futures::future::BoxFuture;
use futures::lock::Mutex;
use futures::prelude::*;
use futures_boxed::boxed;
use jsonrpc::client::{FutureResult, Result};
use jsonrpc_derive::{jsonrpc_client, jsonrpc_method};
use lsp_types::*;
use std::borrow::Cow;

#[jsonrpc_client(LatexLspClient)]
pub trait LspClient {
    #[jsonrpc_method("workspace/configuration", kind = "request")]
    fn configuration(&self, params: ConfigurationParams) -> FutureResult<'_, serde_json::Value>;

    #[jsonrpc_method("window/showMessage", kind = "notification")]
    fn show_message(&self, params: ShowMessageParams) -> BoxFuture<'_, ()>;

    #[jsonrpc_method("client/registerCapability", kind = "request")]
    fn register_capability(&self, params: RegistrationParams) -> FutureResult<'_, ()>;

    #[jsonrpc_method("textDocument/publishDiagnostics", kind = "notification")]
    fn publish_diagnostics(&self, params: PublishDiagnosticsParams) -> BoxFuture<'_, ()>;
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LspClientMockOptions {
    pub bibtex_formatting: Option<BibtexFormattingOptions>,
}

#[derive(Debug, Default)]
pub struct LspClientMock {
    pub messages: Mutex<Vec<ShowMessageParams>>,
    pub options: Mutex<LspClientMockOptions>,
}

impl LspClient for LspClientMock {
    #[boxed]
    async fn configuration(&self, params: ConfigurationParams) -> Result<serde_json::Value> {
        let options = self.options.lock().await;
        match params.items[0].section {
            Some(Cow::Borrowed("bibtex.formatting")) => options
                .bibtex_formatting
                .as_ref()
                .map(|options| serde_json::to_value(vec![options]).unwrap())
                .ok_or(jsonrpc::Error::internal_error("Internal error".to_owned())),
            _ => {
                unreachable!();
            }
        }
    }

    #[boxed]
    async fn show_message(&self, params: ShowMessageParams) {
        let mut messages = self.messages.lock().await;
        messages.push(params);
    }

    #[boxed]
    async fn register_capability(&self, _params: RegistrationParams) -> Result<()> {
        Ok(())
    }

    #[boxed]
    async fn publish_diagnostics(&self, _params: PublishDiagnosticsParams) {}
}
