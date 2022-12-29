use std::{env, ffi::OsStr, path::PathBuf, process::Command};

use anyhow::Result;

pub fn root_directories() -> Result<Vec<PathBuf>> {
    let texmf = run(&["-var-value", "TEXMF"])?;
    let expand_arg = format!("--expand-braces={}", texmf);
    let expanded = run(&[&expand_arg])?;
    let directories = env::split_paths(&expanded.replace('!', ""))
        .filter(|path| path.exists())
        .collect();
    Ok(directories)
}

fn run(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<String> {
    let output = Command::new("kpsewhich").args(args).output()?;

    let result = String::from_utf8(output.stdout)?
        .lines()
        .next()
        .unwrap()
        .into();

    Ok(result)
}
