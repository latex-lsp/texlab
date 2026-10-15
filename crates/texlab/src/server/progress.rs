use lsp_types::{
    NumberOrString, ProgressParams, ProgressParamsValue, WorkDoneProgress, WorkDoneProgressBegin,
    WorkDoneProgressCreateParams, WorkDoneProgressEnd, notification::Progress,
    request::WorkDoneProgressCreate,
};

use crate::{LspClient, util::from_proto};

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
    pub fn new(client: LspClient, token: i32, uri: &lsp_types::Uri) -> Self {
        let _ = client.send_request::<WorkDoneProgressCreate>(WorkDoneProgressCreateParams {
            token: NumberOrString::Number(token),
        });

        let _ = client.send_notification::<Progress>(ProgressParams {
            token: NumberOrString::Number(token),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(WorkDoneProgressBegin {
                title: "Building".into(),
                message: Some(display_path(uri)),
                cancellable: Some(false),
                percentage: None,
            })),
        });

        Self { client, token }
    }

    pub fn new_inputs_progress(client: LspClient, token: i32) -> Self {
        let _ = client.send_request::<WorkDoneProgressCreate>(WorkDoneProgressCreateParams {
            token: NumberOrString::Number(token),
        });

        let _ = client.send_notification::<Progress>(ProgressParams {
            token: NumberOrString::Number(token),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(WorkDoneProgressBegin {
                title: "Parsing Dependencies.".into(),
                message: None,
                cancellable: Some(false),
                percentage: None,
            })),
        });

        Self { client, token }
    }
}

fn display_path(uri: &lsp_types::Uri) -> String {
    match from_proto::url(uri).to_file_path() {
        Ok(path) => path.display().to_string(),
        Err(_) => String::from(uri.as_str()),
    }
}
