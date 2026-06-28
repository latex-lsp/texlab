use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use anyhow::Result;
use base_db::Workspace;
use crossbeam_channel::Sender;
use lsp_types::{PublishDiagnosticsParams, notification::PublishDiagnostics};
use rustc_hash::FxHashSet;
use threadpool::ThreadPool;
use url::Url;

use crate::{client::LspClient, util::to_proto};

use super::InternalMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PublishToken(u64);

#[derive(Debug, Default)]
pub(super) struct DiagnosticsPublisher {
    next_token: AtomicU64,
    pending_token: Option<PublishToken>,
    published_diagnostics: FxHashSet<Url>,
}

impl DiagnosticsPublisher {
    pub(super) fn schedule(
        &mut self,
        sender: &Sender<InternalMessage>,
        pool: &ThreadPool,
        delay: Duration,
    ) {
        let token = self.schedule_token();
        let sender = sender.clone();

        pool.execute(move || {
            std::thread::sleep(delay);
            sender.send(InternalMessage::Diagnostics(token)).unwrap();
        });
    }

    pub(super) fn should_publish(&mut self, token: PublishToken) -> bool {
        if self.pending_token == Some(token) {
            self.pending_token = None;
            true
        } else {
            false
        }
    }

    pub(super) fn publish(
        &mut self,
        client: &LspClient,
        workspace: &Workspace,
        diagnostic_manager: &diagnostics::Manager,
    ) -> Result<()> {
        for params in self.collect(workspace, diagnostic_manager) {
            client.send_notification::<PublishDiagnostics>(params)?;
        }

        Ok(())
    }

    fn schedule_token(&mut self) -> PublishToken {
        let token = PublishToken(self.next_token.fetch_add(1, Ordering::Relaxed));
        self.pending_token = Some(token);
        token
    }

    fn collect(
        &mut self,
        workspace: &Workspace,
        diagnostic_manager: &diagnostics::Manager,
    ) -> Vec<PublishDiagnosticsParams> {
        let mut params = Vec::new();
        let current_diagnostics = diagnostic_manager.get(workspace);
        let current_uris = current_diagnostics
            .keys()
            .cloned()
            .collect::<FxHashSet<_>>();

        for (uri, diagnostics) in current_diagnostics {
            let Some(document) = workspace.lookup(&uri) else {
                continue;
            };

            let diagnostics = diagnostics
                .into_iter()
                .filter_map(|diagnostic| to_proto::diagnostic(workspace, document, &diagnostic))
                .collect();

            params.push(PublishDiagnosticsParams {
                uri: to_proto::uri(&uri),
                diagnostics,
                version: None,
            });
        }

        for uri in self.published_diagnostics.difference(&current_uris) {
            params.push(PublishDiagnosticsParams {
                uri: to_proto::uri(uri),
                diagnostics: Vec::new(),
                version: None,
            });
        }

        self.published_diagnostics = current_uris;
        params
    }
}

#[cfg(test)]
mod tests {
    use super::DiagnosticsPublisher;

    #[test]
    fn test_stale_scheduled_publishes_are_ignored() {
        let mut publisher = DiagnosticsPublisher::default();
        let first = publisher.schedule_token();
        let second = publisher.schedule_token();

        assert!(!publisher.should_publish(first));
        assert!(publisher.should_publish(second));
    }

    #[test]
    fn test_pending_publish_is_consumed_once() {
        let mut publisher = DiagnosticsPublisher::default();
        let token = publisher.schedule_token();

        assert!(publisher.should_publish(token));
        assert!(!publisher.should_publish(token));
    }
}
