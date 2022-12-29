use std::sync::atomic::{AtomicI32, Ordering};

use anyhow::Result;
use lsp_types::{
    notification::Progress, request::WorkDoneProgressCreate, NumberOrString, ProgressParams,
    ProgressParamsValue, Url, WorkDoneProgress, WorkDoneProgressBegin,
    WorkDoneProgressCreateParams, WorkDoneProgressEnd,
};

use crate::client::LspClient;

static NEXT_TOKEN: AtomicI32 = AtomicI32::new(1);

pub struct Reporter<'a> {
    client: &'a LspClient,
    token: i32,
}

impl<'a> Reporter<'a> {
    pub fn new(client: &'a LspClient) -> Self {
        let token = NEXT_TOKEN.fetch_add(1, Ordering::SeqCst);
        Self { client, token }
    }

    pub fn start(&self, uri: &Url) -> Result<()> {
        self.client
            .send_request::<WorkDoneProgressCreate>(WorkDoneProgressCreateParams {
                token: NumberOrString::Number(self.token),
            })?;

        self.client.send_notification::<Progress>(ProgressParams {
            token: NumberOrString::Number(self.token),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(WorkDoneProgressBegin {
                title: "Building".to_string(),
                message: Some(uri.as_str().to_string()),
                cancellable: Some(false),
                percentage: None,
            })),
        })?;

        Ok(())
    }
}

impl<'a> Drop for Reporter<'a> {
    fn drop(&mut self) {
        let _ = self.client.send_notification::<Progress>(ProgressParams {
            token: NumberOrString::Number(self.token),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(WorkDoneProgressEnd {
                message: None,
            })),
        });
    }
}
