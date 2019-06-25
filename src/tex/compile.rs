use futures::compat::*;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::{tempdir, TempDir};
use wait_timeout::ChildExt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Format {
    Latex,
    Lualatex,
    Xelatex,
}

impl From<&Path> for Format {
    fn from(file: &Path) -> Self {
        match file.to_string_lossy().as_ref() {
            file if file.contains("lua") => Format::Lualatex,
            file if file.contains("xe") => Format::Xelatex,
            _ => Format::Latex,
        }
    }
}

#[derive(Debug)]
pub struct CompilationResult {
    pub log: String,
    pub directory: TempDir,
}

pub async fn compile<'a>(
    file_name: &'a str,
    code: &'a str,
    format: Format,
) -> io::Result<CompilationResult> {
    let directory = tempdir()?;
    let tex_file = directory.path().join(file_name);
    tokio::fs::write(tex_file.clone(), code).compat().await?;

    let executable = match format {
        Format::Latex => "latex",
        Format::Lualatex => "lualatex",
        Format::Xelatex => "xelatex",
    };
    let mut process = Command::new(executable)
        .args(&["--interaction=batchmode", "-shell-escape", file_name])
        .current_dir(&directory)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    match process.wait_timeout(Duration::from_secs(10))? {
        Some(_) => {
            let log_file = tex_file.with_extension("log");
            let bytes = tokio::fs::read(log_file).compat().await?;
            let log = String::from_utf8_lossy(&bytes).into_owned();
            Ok(CompilationResult { log, directory })
        }
        None => {
            process.kill()?;
            Err(io::Error::from(io::ErrorKind::TimedOut))
        }
    }
}
