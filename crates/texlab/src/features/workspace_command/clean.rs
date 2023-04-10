use std::process::Stdio;

use anyhow::Result;
use base_db::Workspace;
use lsp_types::{TextDocumentIdentifier, Url};
use thiserror::Error;

use crate::normalize_uri;

#[derive(Debug, Error)]
pub enum CleanError {
    #[error("document '{0}' not found")]
    DocumentNotFound(Url),

    #[error("document '{0}' is not a local file")]
    NoLocalFile(Url),

    #[error("document was not provided as an argument")]
    MissingArg,

    #[error("invalid argument: {0}")]
    InvalidArg(serde_json::Error),

    #[error("failed to spawn process: {0}")]
    Spawn(std::io::Error),
}

#[derive(Debug)]
pub struct CleanCommand {
    executable: String,
    args: Vec<String>,
}

impl CleanCommand {
    pub fn new(
        workspace: &Workspace,
        options: CleanOptions,
        args: Vec<serde_json::Value>,
    ) -> Result<Self> {
        let params: TextDocumentIdentifier =
            serde_json::from_value(args.into_iter().next().ok_or(CleanError::MissingArg)?)
                .map_err(CleanError::InvalidArg)?;

        let mut uri = params.uri;
        normalize_uri(&mut uri);

        let document = workspace
            .lookup(&uri)
            .ok_or_else(|| CleanError::DocumentNotFound(uri.clone()))?;

        let path = document
            .path
            .as_deref()
            .ok_or_else(|| CleanError::NoLocalFile(uri.clone()))?;

        let current_dir = workspace.current_dir(&document.dir);

        let output_dir = workspace.output_dir(&current_dir).to_file_path().unwrap();

        let flag = match options {
            CleanOptions::Auxiliary => "-c",
            CleanOptions::Artifacts => "-C",
        };

        let executable = "latexmk".to_string();
        let args = vec![
            format!("-outdir={}", output_dir.display()),
            flag.to_string(),
            path.display().to_string(),
        ];

        Ok(Self { executable, args })
    }

    pub fn run(self) -> Result<()> {
        log::debug!("Cleaning output files: {} {:?}", self.executable, self.args);
        std::process::Command::new(self.executable)
            .args(self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(move |msg| anyhow::Error::new(CleanError::Spawn(msg)))?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum CleanOptions {
    Auxiliary,
    Artifacts,
}
