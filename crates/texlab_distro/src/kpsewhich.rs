use super::language::Language;
use super::{LoadError, Resolver};
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub async fn parse_database<R>(reader: R) -> Result<Resolver, LoadError>
where
    R: Fn(&Path) -> Result<Vec<PathBuf>, LoadError>,
{
    let root_directories = root_directories().await?;
    let mut files_by_name = HashMap::new();
    for directory in &root_directories {
        let database = reader(directory)?
            .into_iter()
            .filter(|path| {
                path.extension()
                    .and_then(OsStr::to_str)
                    .and_then(Language::by_extension)
                    .is_some()
            })
            .filter_map(|path| {
                root_directories
                    .iter()
                    .rev()
                    .find_map(move |dir| dir.join(&path).canonicalize().ok())
            })
            .map(|path| (path.file_name().unwrap().to_str().unwrap().to_owned(), path));

        files_by_name.extend(database);
    }
    Ok(Resolver::new(files_by_name))
}

async fn root_directories() -> Result<Vec<PathBuf>, LoadError> {
    let texmf = run(&["-var-value", "TEXMF"]).await?;
    let expand_arg = format!("--expand-braces={}", texmf);
    let expanded = run(&[&expand_arg]).await?;
    let directories = env::split_paths(&expanded.replace("!", ""))
        .filter(|path| path.exists())
        .collect();
    Ok(directories)
}

async fn run<I, S>(args: I) -> Result<String, LoadError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("kpsewhich")
        .args(args)
        .output()
        .await
        .map_err(|_| LoadError::KpsewhichNotFound)?;

    let result = String::from_utf8(output.stdout)
        .expect("Could not decode output from kpsewhich")
        .lines()
        .next()
        .expect("Invalid output from kpsewhich")
        .into();

    Ok(result)
}
