mod progress;

use std::{
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::Stdio,
    thread::{self, JoinHandle},
};

use encoding_rs_io::DecodeReaderBytesBuilder;
use lsp_types::{notification::LogMessage, LogMessageParams, TextDocumentIdentifier, Url};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{client::LspClient, db::Workspace, util::capabilities::ClientCapabilitiesExt, Db};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResult {
    pub status: BuildStatus,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum BuildStatus {
    SUCCESS = 0,
    ERROR = 1,
    FAILURE = 2,
    CANCELLED = 3,
}

#[derive(Debug)]
pub struct Command {
    uri: Url,
    progress: bool,
    executable: String,
    args: Vec<String>,
    working_dir: PathBuf,
    client: LspClient,
}

impl Command {
    pub fn new(db: &dyn Db, uri: Url, client: LspClient) -> Option<Self> {
        let workspace = Workspace::get(db);
        let document = match workspace.lookup_uri(db, &uri) {
            Some(child) => workspace
                .parents(db, child)
                .iter()
                .next()
                .copied()
                .unwrap_or(child),
            None => return None,
        };

        if document.location(db).path(db).is_none() {
            log::warn!("Document {uri} cannot be compiled; skipping...");
            return None;
        }

        let options = &workspace.options(db).build;
        let executable = options.executable.0.clone();
        let path = document.location(db).path(db).as_deref().unwrap();
        let args = options
            .args
            .0
            .iter()
            .map(|arg| replace_placeholder(arg, path))
            .collect();

        let working_dir = workspace
            .working_dir(db, document.directory(db))
            .path(db)
            .clone()?;

        Some(Self {
            uri: document.location(db).uri(db).clone(),
            progress: workspace
                .client_capabilities(db)
                .has_work_done_progress_support(),
            executable,
            args,
            working_dir,
            client,
        })
    }

    pub fn run(self) -> BuildStatus {
        let reporter = if self.progress {
            let inner = progress::Reporter::new(&self.client);
            inner.start(&self.uri).expect("report progress");
            Some(inner)
        } else {
            None
        };

        let mut process = match std::process::Command::new(&self.executable)
            .args(self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&self.working_dir)
            .spawn()
        {
            Ok(process) => process,
            Err(why) => {
                log::error!(
                    "Failed to spawn process {:?} in directory {}: {}",
                    self.executable,
                    self.working_dir.display(),
                    why
                );
                return BuildStatus::FAILURE;
            }
        };

        let (line_sender, line_receiver) = flume::unbounded();
        let (exit_sender, exit_receiver) = flume::unbounded();
        track_output(process.stderr.take().unwrap(), line_sender.clone());
        track_output(process.stdout.take().unwrap(), line_sender);
        let client = self.client.clone();
        let handle = std::thread::spawn(move || {
            let typ = lsp_types::MessageType::LOG;

            loop {
                let done = flume::Selector::new()
                    .recv(&line_receiver, |line| match line {
                        Ok(message) => {
                            let params = LogMessageParams { message, typ };
                            client.send_notification::<LogMessage>(params).unwrap();
                            false
                        }
                        Err(_) => true,
                    })
                    .recv(&exit_receiver, |_| true)
                    .wait();

                if done {
                    break;
                }
            }
        });

        let status = process.wait().map_or(BuildStatus::FAILURE, |result| {
            if result.success() {
                BuildStatus::SUCCESS
            } else {
                BuildStatus::ERROR
            }
        });

        let _ = exit_sender.send(());
        handle.join().unwrap();

        drop(reporter);
        status
    }
}

fn track_output(
    output: impl Read + Send + 'static,
    sender: flume::Sender<String>,
) -> JoinHandle<()> {
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

fn replace_placeholder(arg: &str, file: &Path) -> String {
    if arg.starts_with('"') || arg.ends_with('"') {
        arg.to_string()
    } else {
        arg.replace("%f", &file.to_string_lossy())
    }
}
