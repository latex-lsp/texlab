use std::{
    io::{self, BufRead, BufReader, Read},
    path::Path,
    process::{Command, Stdio},
    thread,
};

use cancellation::CancellationToken;
use crossbeam_channel::Sender;
use encoding_rs_io::DecodeReaderBytesBuilder;
use lsp_types::TextDocumentIdentifier;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::Options;

use super::FeatureRequest;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum BuildStatus {
    Success = 0,
    Error = 1,
    Failure = 2,
    Cancelled = 3,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResult {
    pub status: BuildStatus,
}

pub fn build_document(
    request: FeatureRequest<BuildParams>,
    _cancellation_token: &CancellationToken,
    log_sender: Sender<String>,
) -> BuildResult {
    let document = request
        .subset
        .documents
        .iter()
        .find(|document| {
            if let Some(data) = document.data.as_latex() {
                data.extras.has_document_environment
            } else {
                false
            }
        })
        .map(|document| document.as_ref())
        .unwrap_or_else(|| request.main_document());

    if document.uri.scheme() != "file" {
        return BuildResult {
            status: BuildStatus::Failure,
        };
    }
    log::info!("Building document {}", document.uri.as_str());

    let options = { request.context.options.read().unwrap().clone() };

    let status = match build_internal(&document.uri.to_file_path().unwrap(), &options, log_sender) {
        Ok(true) => BuildStatus::Success,
        Ok(false) => BuildStatus::Error,
        Err(why) => {
            log::error!("Failed to execute textDocument/build: {}", why);
            BuildStatus::Failure
        }
    };
    BuildResult { status }
}

fn build_internal(path: &Path, options: &Options, log_sender: Sender<String>) -> io::Result<bool> {
    let build_dir = options
        .root_directory
        .as_ref()
        .map(AsRef::as_ref)
        .or_else(|| path.parent())
        .unwrap();

    let args: Vec<_> = options
        .build
        .args()
        .into_iter()
        .map(|arg| replace_placeholder(arg, path))
        .collect();

    let mut process = Command::new(options.build.executable())
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(build_dir)
        .spawn()?;

    track_output(process.stdout.take().unwrap(), log_sender.clone());
    track_output(process.stderr.take().unwrap(), log_sender.clone());

    if !options.build.is_continuous {
        process.wait().map(|status| status.success())
    } else {
        Ok(true)
    }
}

fn replace_placeholder(arg: String, file: &Path) -> String {
    if arg.starts_with('"') || arg.ends_with('"') {
        arg
    } else {
        arg.replace("%f", &file.to_string_lossy())
    }
}

fn track_output(output: impl Read + Send + 'static, sender: Sender<String>) {
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
    });
}
