use futures::compat::*;
use std::process::{Command, Stdio};
use tempfile::{tempdir, TempDir};
use wait_timeout::ChildExt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Format {
    Latex,
    Pdflatex,
    Lualatex,
    Xelatex,
}

impl Format {
    pub fn executable(&self) -> &'static str {
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

    let mut process = Command::new(format.executable())
        .args(&["--interaction=batchmode", "-shell-escape", file_name])
        .current_dir(&directory)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| CompileError::LatexNotInstalled)?;

    match process
        .wait_timeout_ms(10_000)
        .map_err(|_| CompileError::Wait)?
    {
        Some(_) => {
            let log_file = code_file.with_extension("log");
            let log_bytes = tokio::fs::read(log_file)
                .compat()
                .await
                .map_err(|_| CompileError::ReadLog)?;
            let log = String::from_utf8_lossy(&log_bytes).into_owned();
            Ok(CompileResult { log, directory })
        }
        None => {
            process.kill().map_err(|_| CompileError::Cleanup)?;
            Err(CompileError::Timeout)
        }
    }
}
