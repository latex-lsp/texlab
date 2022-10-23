use std::{
    io::{BufRead, BufReader, Read},
    path::Path,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use dashmap::DashMap;
use encoding_rs_io::DecodeReaderBytesBuilder;
use lsp_types::{
    notification::{LogMessage, Progress},
    LogMessageParams, NumberOrString, Position, ProgressParams, ProgressParamsValue,
    TextDocumentIdentifier, Url, WorkDoneProgress, WorkDoneProgressBegin,
    WorkDoneProgressCreateParams, WorkDoneProgressEnd,
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use uuid::Uuid;

use crate::{client::LspClient, ClientCapabilitiesExt};

use super::{FeatureRequest, ForwardSearch};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum BuildStatus {
    SUCCESS = 0,
    ERROR = 1,
    FAILURE = 2,
    CANCELLED = 3,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResult {
    pub status: BuildStatus,
}

struct ProgressReporter<'a> {
    supports_progress: bool,
    client: LspClient,
    token: &'a str,
}

impl<'a> ProgressReporter<'a> {
    pub fn start(&self, uri: &Url) -> Result<()> {
        if self.supports_progress {
            self.client
                .send_request::<lsp_types::request::WorkDoneProgressCreate>(
                    WorkDoneProgressCreateParams {
                        token: NumberOrString::String(self.token.to_string()),
                    },
                )?;

            self.client.send_notification::<Progress>(ProgressParams {
                token: NumberOrString::String(self.token.to_string()),
                value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(
                    WorkDoneProgressBegin {
                        title: "Building".to_string(),
                        message: Some(uri.as_str().to_string()),
                        cancellable: Some(false),
                        percentage: None,
                    },
                )),
            })?;
        };
        Ok(())
    }
}

impl<'a> Drop for ProgressReporter<'a> {
    fn drop(&mut self) {
        if self.supports_progress {
            drop(self.client.send_notification::<Progress>(ProgressParams {
                token: NumberOrString::String(self.token.to_string()),
                value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(WorkDoneProgressEnd {
                    message: None,
                })),
            }));
        }
    }
}

#[derive(Default)]
pub struct BuildEngine {
    lock: Mutex<()>,
    pub positions_by_uri: DashMap<Arc<Url>, Position>,
}

impl BuildEngine {
    pub fn build(
        &self,
        request: FeatureRequest<BuildParams>,
        client: LspClient,
    ) -> Result<BuildResult> {
        let lock = self.lock.lock().unwrap();

        let document = request
            .workspace
            .iter()
            .find(|document| {
                document
                    .data()
                    .as_latex()
                    .map_or(false, |data| data.extras.has_document_environment)
            })
            .unwrap_or_else(|| request.main_document());

        if !document.can_be_compiled() {
            log::info!(
                "Document {} cannot be compiled; skipping...",
                document.uri()
            );

            return Ok(BuildResult {
                status: BuildStatus::SUCCESS,
            });
        }

        if document.uri().scheme() != "file" {
            return Ok(BuildResult {
                status: BuildStatus::FAILURE,
            });
        }

        let path = document.uri().to_file_path().unwrap();

        let supports_progress = request
            .workspace
            .environment
            .client_capabilities
            .has_work_done_progress_support();

        let token = format!("texlab-build-{}", Uuid::new_v4());
        let progress_reporter = ProgressReporter {
            supports_progress,
            client: client.clone(),
            token: &token,
        };
        progress_reporter.start(document.uri())?;

        let options = &request.workspace.environment.options;

        let build_dir = options
            .root_directory
            .as_ref()
            .map(AsRef::as_ref)
            .or_else(|| path.parent())
            .unwrap();

        let args: Vec<_> = options
            .build
            .args
            .0
            .iter()
            .map(|arg| replace_placeholder(arg.clone(), &path))
            .collect();

        let mut process = Command::new(&options.build.executable.0)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(build_dir)
            .spawn()?;

        let (exit_sender, exit_receiver) = crossbeam_channel::bounded(1);
        let log_handle = capture_output(&mut process, client, exit_receiver);
        let success = process.wait().map(|status| status.success())?;
        exit_sender.send(())?;
        drop(exit_sender);

        log_handle.join().unwrap();
        let status = if success {
            BuildStatus::SUCCESS
        } else {
            BuildStatus::ERROR
        };

        drop(progress_reporter);
        drop(lock);

        if let Some((executable, args)) = options
            .forward_search
            .executable
            .as_deref()
            .zip(options.forward_search.args.as_deref())
            .filter(|_| options.build.forward_search_after)
        {
            let position = self
                .positions_by_uri
                .get(&request.uri)
                .map(|entry| *entry.value())
                .unwrap_or_default();

            ForwardSearch::builder()
                .executable(executable)
                .args(args)
                .line(position.line)
                .workspace(&request.workspace)
                .tex_uri(&request.uri)
                .build()
                .execute();
        }

        Ok(BuildResult { status })
    }
}

fn capture_output(
    process: &mut std::process::Child,
    client: LspClient,
    exit_receiver: Receiver<()>,
) -> JoinHandle<()> {
    let (log_sender, log_receiver) = crossbeam_channel::unbounded();
    track_output(process.stdout.take().unwrap(), log_sender.clone());
    track_output(process.stderr.take().unwrap(), log_sender);
    thread::spawn(move || loop {
        crossbeam_channel::select! {
            recv(&log_receiver) -> message => {
                if let Ok(message) = message {
                    client.send_notification::<LogMessage>(
                        LogMessageParams {
                            message,
                            typ: lsp_types::MessageType::LOG,
                        },
                    )
                    .unwrap();
                }
            },
            recv(&exit_receiver) -> _ => break,
        };
    })
}

fn replace_placeholder(arg: String, file: &Path) -> String {
    if arg.starts_with('"') || arg.ends_with('"') {
        arg
    } else {
        arg.replace("%f", &file.to_string_lossy())
    }
}

fn track_output(output: impl Read + Send + 'static, sender: Sender<String>) -> JoinHandle<()> {
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::UTF_8))
            .utf8_passthru(true)
            .strip_bom(true)
            .build(output),
    );

    thread::spawn(move || {
        for line in reader.lines() {
            sender.send(line.unwrap()).unwrap();
        }
    })
}
