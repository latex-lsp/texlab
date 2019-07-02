use crate::build::BuildOptions;
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

    #[jsonrpc_method("window/progress/start", kind = "notification")]
    #[boxed]
    async fn progress_start(&self, params: ProgressStartParams) -> ();

    #[jsonrpc_method("window/progress/report", kind = "notification")]
    #[boxed]
    async fn progress_report(&self, params: ProgressReportParams) -> ();

    #[jsonrpc_method("window/progress/done", kind = "notification")]
    #[boxed]
    async fn progress_done(&self, params: ProgressDoneParams) -> ();

    #[jsonrpc_method("window/logMessage", kind = "notification")]
    #[boxed]
    async fn log_message(&self, params: LogMessageParams) -> ();
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LspClientMockOptions {
    pub bibtex_formatting: Option<BibtexFormattingOptions>,
    pub latex_lint: Option<LatexLintOptions>,
    pub latex_build: Option<BuildOptions>,
}

#[derive(Debug, Default)]
pub struct LspClientMock {
    pub messages: Mutex<Vec<ShowMessageParams>>,
    pub options: Mutex<LspClientMockOptions>,
    pub diagnostics_by_uri: Mutex<HashMap<Uri, Vec<Diagnostic>>>,
    pub log_messages: Mutex<Vec<LogMessageParams>>,
}

impl LspClientMock {
    pub async fn log(&self) -> String {
        let messages = self.log_messages.lock().await;
        let mut combined_message = String::new();
        for params in messages.iter() {
            combined_message.push_str(&params.message);
            combined_message.push('\n');
        }
        combined_message
    }
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
            Some(Cow::Borrowed("latex.build")) => serialize(&options.latex_build),
            _ => panic!("Invalid language configuration!"),
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
    async fn progress_start(&self, _params: ProgressStartParams) {}

    #[boxed]
    async fn progress_report(&self, _params: ProgressReportParams) {}

    #[boxed]
    async fn progress_done(&self, _params: ProgressDoneParams) {}

    #[boxed]
    async fn log_message(&self, params: LogMessageParams) {
        let mut messages = self.log_messages.lock().await;
        messages.push(params);
    }
}
