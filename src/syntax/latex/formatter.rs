use std::{io, process::Stdio};
use tempfile::tempdir;
use tokio::fs;
use tokio::process::Command;

pub async fn format(text: &str) -> io::Result<String> {
    let dir = tempdir()?;
    fs::write(dir.path().join("input.tex"), text).await?;

    Command::new("latexindent")
        .args(&["-o", "output.tex", "input.tex"])
        .current_dir(dir.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .await?;

    fs::read_to_string(dir.path().join("output.tex")).await
}
