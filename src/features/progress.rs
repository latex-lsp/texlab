use std::sync::atomic::{AtomicI32, Ordering};

use anyhow::Result;
use lsp_types::{
    notification::Progress, NumberOrString, ProgressParams, ProgressParamsValue, WorkDoneProgress,
    WorkDoneProgressBegin, WorkDoneProgressCreateParams, WorkDoneProgressEnd,
};

use crate::client::LspClient;

static NEXT_TOKEN: AtomicI32 = AtomicI32::new(1);

pub struct ProgressReporter<'a> {
    client: &'a LspClient,
    token: i32,
}

impl<'a> ProgressReporter<'a> {
    pub fn new(client: &'a LspClient, title: String, message: String) -> Result<Self> {
        let token = NEXT_TOKEN.fetch_add(1, Ordering::SeqCst);

        client.send_request::<lsp_types::request::WorkDoneProgressCreate>(
            WorkDoneProgressCreateParams {
                token: NumberOrString::Number(token),
            },
        )?;

        client.send_notification::<Progress>(ProgressParams {
            token: NumberOrString::Number(token),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(WorkDoneProgressBegin {
                title,
                message: Some(message),
                cancellable: Some(false),
                percentage: None,
            })),
        })?;

        Ok(Self { client, token })
    }
}

impl<'a> Drop for ProgressReporter<'a> {
    fn drop(&mut self) {
        let _ = self.client.send_notification::<Progress>(ProgressParams {
            token: NumberOrString::String(self.token.to_string()),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(WorkDoneProgressEnd {
                message: None,
            })),
        });
    }
}
