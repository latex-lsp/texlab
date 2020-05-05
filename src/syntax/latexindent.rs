use std::{io, process::Stdio};
use tempfile::tempdir;
use tokio::{fs, process::Command};

pub async fn format(text: &str, extension: &str) -> io::Result<String> {
    let dir = tempdir()?;
    let input = format!("input.{}", extension);
    let output = format!("output.{}", extension);
    fs::write(dir.path().join(&input), text).await?;

    Command::new("latexindent")
        .args(&["-o", &output, &input])
        .current_dir(dir.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .await?;

    fs::read_to_string(dir.path().join(&output)).await
}
