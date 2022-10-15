use std::{
    io::{BufRead, BufReader, Read},
    path::Path,
    process::{Command, Stdio},
    thread::JoinHandle,
};

use anyhow::{Ok, Result};
use encoding_rs_io::DecodeReaderBytesBuilder;
use lsp_types::{notification::LogMessage, LogMessageParams, TextDocumentIdentifier, Url};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use typed_builder::TypedBuilder;

use crate::{client::LspClient, DocumentLanguage, Workspace};

use super::progress::ProgressReporter;

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

#[derive(TypedBuilder)]
pub struct BuildRunner<'a> {
    executable: &'a str,
    args: &'a [String],
    workspace: &'a Workspace,
    tex_uri: &'a Url,
    client: &'a LspClient,
    report_progress: bool,
}

impl<'a> BuildRunner<'a> {
    pub fn run(self) -> Result<BuildStatus> {
        let path = match self
            .workspace
            .iter()
            .find(|document| {
                document
                    .data()
                    .as_latex()
                    .map_or(false, |document| document.extras.can_be_built)
            })
            .or_else(|| self.workspace.get(self.tex_uri))
            .filter(|document| document.data().language() == DocumentLanguage::Latex)
            .filter(|document| document.uri().scheme() == "file")
            .and_then(|document| document.uri().to_file_path().ok())
        {
            Some(path) => path,
            None => return Ok(BuildStatus::FAILURE),
        };

        let reporter = if self.report_progress {
            Some(ProgressReporter::new(
                self.client,
                "Building".to_string(),
                path.display().to_string(),
            )?)
        } else {
            None
        };

        let args: Vec<_> = self
            .args
            .iter()
            .map(|arg| replace_placeholder(arg.clone(), &path))
            .collect();

        let mut process = Command::new(self.executable)
            .args(args)
            .current_dir(path.parent().unwrap())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .stdin(Stdio::null())
            .spawn()?;

        self.track_output(process.stdout.take().unwrap());
        self.track_output(process.stderr.take().unwrap());

        let status = if process.wait()?.success() {
            BuildStatus::SUCCESS
        } else {
            BuildStatus::FAILURE
        };

        drop(reporter);
        Ok(status)
    }

    fn track_output(&self, output: impl Read + Send + 'static) -> JoinHandle<()> {
        let client = self.client.clone();
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(encoding_rs::UTF_8))
                .utf8_passthru(true)
                .strip_bom(true)
                .build(output),
        );

        std::thread::spawn(move || {
            for line in reader.lines() {
                let message = line.unwrap();
                let typ = lsp_types::MessageType::LOG;
                client
                    .send_notification::<LogMessage>(LogMessageParams { message, typ })
                    .unwrap();
            }
        })
    }
}

fn replace_placeholder(arg: String, file: &Path) -> String {
    if arg.starts_with('"') || arg.ends_with('"') {
        arg
    } else {
        arg.replace("%f", &file.to_string_lossy())
    }
}
