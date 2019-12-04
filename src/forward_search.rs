use log::*;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::io;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct ForwardSearchOptions {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum ForwardSearchStatus {
    Success = 0,
    Error = 1,
    Failure = 2,
    Unconfigured = 3,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ForwardSearchResult {
    pub status: ForwardSearchStatus,
}

pub async fn search<'a>(
    tex_file: &'a Path,
    parent: &'a Path,
    line_number: u64,
    options: ForwardSearchOptions,
) -> Option<ForwardSearchResult> {
    if options.executable.is_none() || options.args.is_none() {
        return Some(ForwardSearchResult {
            status: ForwardSearchStatus::Unconfigured,
        });
    }

    let pdf_file = parent
        .parent()?
        .join(format!("{}.pdf", parent.file_stem()?.to_str()?));

    let args: Vec<String> = options
        .args
        .unwrap()
        .into_iter()
        .flat_map(|arg| replace_placeholder(&tex_file, &pdf_file, line_number, arg))
        .collect();

    let status = match spawn_process(options.executable.unwrap(), args).await {
        Ok(()) => ForwardSearchStatus::Success,
        Err(why) => {
            error!("Unable to execute forward search: {}", why);
            ForwardSearchStatus::Failure
        }
    };
    Some(ForwardSearchResult { status })
}

fn replace_placeholder(
    tex_file: &Path,
    pdf_file: &Path,
    line_number: u64,
    argument: String,
) -> Option<String> {
    let result = if argument.starts_with('"') || argument.ends_with('"') {
        argument
    } else {
        argument
            .replace("%f", tex_file.to_str()?)
            .replace("%p", pdf_file.to_str()?)
            .replace("%l", &line_number.to_string())
    };
    Some(result)
}

async fn spawn_process(executable: String, args: Vec<String>) -> io::Result<()> {
    Command::new(executable)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .await?;
    Ok(())
}
