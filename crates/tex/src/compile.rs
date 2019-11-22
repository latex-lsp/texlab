use futures::future::TryFutureExt;
use std::io;
use std::process::Stdio;
use std::time::Duration;
use tempfile::{tempdir, TempDir};
use tokio::fs;
use tokio::future::FutureExt;
use tokio_net::process::Command;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Format {
    Latex,
    Pdflatex,
    Xelatex,
    Lualatex,
}

impl Format {
    pub fn executable(self) -> &'static str {
        match self {
            Self::Latex => "latex",
            Self::Pdflatex => "pdflatex",
            Self::Xelatex => "xelatex",
            Self::Lualatex => "lualatex",
        }
    }
}

#[derive(Debug)]
pub struct CompileResult {
    pub log: String,
    pub directory: TempDir,
}

#[derive(Debug)]
pub enum CompileError {
    IO(io::Error),
    NotInstalled,
    Timeout,
}

impl From<io::Error> for CompileError {
    fn from(error: io::Error) -> Self {
        Self::IO(error)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CompileParams<'a> {
    pub file_name: &'a str,
    pub code: &'a str,
    pub format: Format,
    pub timeout: Duration,
}

pub async fn compile<'a>(
    executable: &'a str,
    args: &'a [&'a str],
    params: CompileParams<'a>,
) -> Result<CompileResult, CompileError> {
    let directory = tempdir()?;
    let code_file = directory.path().join(params.file_name);
    fs::write(code_file.clone(), params.code).await?;

    Command::new(executable)
        .args(args)
        .current_dir(&directory)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| CompileError::NotInstalled)
        .timeout(params.timeout)
        .map_err(|_| CompileError::Timeout)
        .await?
        .map_err(|_| CompileError::NotInstalled)?;

    let log_file = code_file.with_extension("log");
    let log_bytes = fs::read(log_file).await?;
    let log = String::from_utf8_lossy(&log_bytes).into_owned();
    Ok(CompileResult { log, directory })
}
