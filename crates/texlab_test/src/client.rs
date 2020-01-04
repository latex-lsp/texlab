use futures::lock::Mutex;
use futures_boxed::boxed;
use jsonrpc::client::Result;
use serde::Serialize;
use std::collections::HashMap;
use texlab_protocol::*;

#[derive(Debug, Default)]
pub struct MockLspClient {
    pub messages: Mutex<Vec<ShowMessageParams>>,
    pub options: Mutex<Options>,
    pub diagnostics_by_uri: Mutex<HashMap<Uri, Vec<Diagnostic>>>,
    pub log_messages: Mutex<Vec<LogMessageParams>>,
}

impl MockLspClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn verify_no_diagnostics(&self, uri: &Uri) {
        let diagnostics_by_uri = self.diagnostics_by_uri.lock().await;
        assert_eq!(
            diagnostics_by_uri
                .get(uri.into())
                .map(Vec::len)
                .unwrap_or(0),
            0
        );
    }
}

impl LspClient for MockLspClient {
    #[boxed]
    async fn configuration(&self, params: ConfigurationParams) -> Result<serde_json::Value> {
        fn serialize<T>(options: &Option<T>) -> Result<serde_json::Value>
        where
            T: Serialize,
        {
            options
                .as_ref()
                .map(|options| serde_json::to_value(vec![options]).unwrap())
                .ok_or_else(|| jsonrpc::Error::internal_error("Internal error".to_owned()))
        }

        let options = self.options.lock().await;
        match params.items[0].section.as_ref().unwrap().as_ref() {
            "latex" => serialize(&options.latex),
            "bibtex" => serialize(&options.bibtex),
            _ => panic!("invalid language configuration"),
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
        diagnostics_by_uri.insert(params.uri.into(), params.diagnostics);
    }

    #[boxed]
    async fn work_done_progress_create(&self, _params: WorkDoneProgressCreateParams) -> Result<()> {
        Ok(())
    }

    #[boxed]
    async fn progress(&self, _params: ProgressParams) {}

    #[boxed]
    async fn log_message(&self, params: LogMessageParams) {
        let mut messages = self.log_messages.lock().await;
        messages.push(params);
    }
}
