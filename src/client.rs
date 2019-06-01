use crate::diagnostics::LatexLintOptions;
use crate::formatting::bibtex::BibtexFormattingOptions;
use futures::lock::Mutex;
use futures_boxed::boxed;
use jsonrpc::client::Result;
use jsonrpc_derive::{jsonrpc_client, jsonrpc_method};
use lsp_types::*;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;

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

    #[jsonrpc_method("window/progress", kind = "notification")]
    #[boxed]
    async fn progress(&self, params: ProgressParams) -> ();

    #[jsonrpc_method("window/logMessage", kind = "notification")]
    #[boxed]
    async fn log_message(&self, params: LogMessageParams) -> ();
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LspClientMockOptions {
    pub bibtex_formatting: Option<BibtexFormattingOptions>,
    pub latex_lint: Option<LatexLintOptions>,
}

#[derive(Debug, Default)]
pub struct LspClientMock {
    pub messages: Mutex<Vec<ShowMessageParams>>,
    pub options: Mutex<LspClientMockOptions>,
    pub diagnostics_by_uri: Mutex<HashMap<Uri, Vec<Diagnostic>>>,
}

impl LspClient for LspClientMock {
    #[boxed]
    async fn configuration(&self, params: ConfigurationParams) -> Result<serde_json::Value> {
        fn serialize<T>(options: &Option<T>) -> Result<serde_json::Value>
        where
            T: Serialize,
        {
            options
                .as_ref()
                .map(|options| serde_json::to_value(vec![options]).unwrap())
                .ok_or(jsonrpc::Error::internal_error("Internal error".to_owned()))
        }

        let options = self.options.lock().await;
        match params.items[0].section {
            Some(Cow::Borrowed("bibtex.formatting")) => serialize(&options.bibtex_formatting),
            Some(Cow::Borrowed("latex.lint")) => serialize(&options.latex_lint),
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
    async fn publish_diagnostics(&self, params: PublishDiagnosticsParams) {
        let mut diagnostics_by_uri = self.diagnostics_by_uri.lock().await;
        diagnostics_by_uri.insert(params.uri, params.diagnostics);
    }

    #[boxed]
    async fn progress(&self, _params: ProgressParams) {}

    #[boxed]
    async fn log_message(&self, _params: LogMessageParams) {}
}
