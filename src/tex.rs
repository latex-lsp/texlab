use futures::compat::*;
use futures::future::TryFutureExt;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::{tempdir, TempDir};
use tokio::prelude::FutureExt;
use tokio_process::CommandExt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Format {
    Latex,
    Pdflatex,
    Lualatex,
    Xelatex,
}

impl Format {
    pub fn executable(self) -> &'static str {
        match self {
            Format::Latex => "latex",
            Format::Pdflatex => "pdflatex",
            Format::Lualatex => "lualatex",
            Format::Xelatex => "xelatex",
        }
    }
}

#[derive(Debug)]
pub struct CompileResult {
    pub log: String,
    pub directory: TempDir,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CompileError {
    Initialization,
    LatexNotInstalled,
    Wait,
    ReadLog,
    Cleanup,
    Timeout,
}

pub async fn compile<'a>(
    file_name: &'a str,
    code: &'a str,
    format: Format,
) -> Result<CompileResult, CompileError> {
    let directory = tempdir().map_err(|_| CompileError::Initialization)?;
    let code_file = directory.path().join(file_name);
    tokio::fs::write(code_file.clone(), code)
        .compat()
        .await
        .map_err(|_| CompileError::Initialization)?;

    Command::new(format.executable())
        .args(&["--interaction=batchmode", "-shell-escape", file_name])
        .current_dir(&directory)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status_async()
        .map_err(|_| CompileError::LatexNotInstalled)?
        .timeout(Duration::from_secs(10))
        .compat()
        .map_err(|_| CompileError::Timeout)
        .await?;

    let log_file = code_file.with_extension("log");
    let log_bytes = tokio::fs::read(log_file)
        .compat()
        .await
        .map_err(|_| CompileError::ReadLog)?;
    let log = String::from_utf8_lossy(&log_bytes).into_owned();
    Ok(CompileResult { log, directory })
}
