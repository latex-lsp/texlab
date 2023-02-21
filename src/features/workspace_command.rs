use std::process::Stdio;

use anyhow::{bail, Result};
use lsp_types::{TextDocumentIdentifier, Url};
use thiserror::Error;

use crate::{db::Workspace, normalize_uri, Db};

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
    pub fn new(db: &dyn Db, options: CleanOptions, args: Vec<serde_json::Value>) -> Result<Self> {
        let params: TextDocumentIdentifier =
            serde_json::from_value(args.into_iter().next().ok_or(CleanError::MissingArg)?)
                .map_err(CleanError::InvalidArg)?;

        let mut uri = params.uri;
        normalize_uri(&mut uri);

        let workspace = Workspace::get(db);

        let document = workspace
            .lookup_uri(db, &uri)
            .ok_or_else(|| CleanError::DocumentNotFound(uri.clone()))?;

        let working_dir = workspace.working_dir(db, document.directory(db));

        let output_dir = workspace
            .output_dir(db, working_dir)
            .path(db)
            .as_deref()
            .ok_or_else(|| CleanError::NoLocalFile(uri.clone()))?;

        let path = document
            .location(db)
            .path(db)
            .as_deref()
            .ok_or_else(|| CleanError::NoLocalFile(uri.clone()))?;

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
