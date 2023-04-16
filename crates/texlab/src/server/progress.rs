use lsp_types::{
    notification::Progress, request::WorkDoneProgressCreate, NumberOrString, ProgressParams,
    ProgressParamsValue, Url, WorkDoneProgress, WorkDoneProgressBegin,
    WorkDoneProgressCreateParams, WorkDoneProgressEnd,
};

use crate::LspClient;

#[derive(Debug)]
pub struct ProgressReporter {
    client: LspClient,
    token: i32,
}

impl Drop for ProgressReporter {
    fn drop(&mut self) {
        let _ = self.client.send_notification::<Progress>(ProgressParams {
            token: NumberOrString::Number(self.token),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(WorkDoneProgressEnd {
                message: None,
            })),
        });
    }
}

impl ProgressReporter {
    pub fn new(client: LspClient, token: i32, uri: &Url) -> Self {
        let _ = client.send_request::<WorkDoneProgressCreate>(WorkDoneProgressCreateParams {
            token: NumberOrString::Number(token),
        });

        let _ = client.send_notification::<Progress>(ProgressParams {
            token: NumberOrString::Number(token),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(WorkDoneProgressBegin {
                title: "Building".into(),
                message: Some(String::from(uri.as_str())),
                cancellable: Some(false),
                percentage: None,
            })),
        });

        Self { client, token }
    }
}
