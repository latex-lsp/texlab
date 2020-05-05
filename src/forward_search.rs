use crate::{
    protocol::{ForwardSearchResult, ForwardSearchStatus, Options, Uri},
    workspace::Snapshot,
};
use log::error;
use std::{io, path::Path, process::Stdio};
use tokio::process::Command;

pub async fn search<'a>(
    snapshot: &'a Snapshot,
    tex_uri: &'a Uri,
    line_number: u64,
    options: &Options,
    current_dir: &'a Path,
) -> Option<ForwardSearchResult> {
    let pdf_path = snapshot
        .resolve_aux_targets(
            &snapshot.parent_subfile(tex_uri, options, current_dir)?.uri,
            options,
            current_dir,
            "pdf",
        )?
        .into_iter()
        .filter(|uri| uri.scheme() == "file")
        .filter_map(|uri| uri.to_file_path().ok())
        .find(|path| path.exists())?;

    if tex_uri.scheme() != "file" {
        return Some(ForwardSearchResult {
            status: ForwardSearchStatus::Failure,
        });
    }

    let search_options = options
        .latex
        .as_ref()
        .and_then(|opts| opts.forward_search.as_ref())
        .map(Clone::clone)
        .unwrap_or_default();

    if search_options.executable.is_none() || search_options.args.is_none() {
        return Some(ForwardSearchResult {
            status: ForwardSearchStatus::Unconfigured,
        });
    }

    let tex_path = tex_uri.to_file_path().ok()?;
    let args: Vec<String> = search_options
        .args
        .unwrap()
        .into_iter()
        .flat_map(|arg| replace_placeholder(&tex_path, &pdf_path, line_number, arg))
        .collect();

    let status = match spawn_process(search_options.executable.unwrap(), args).await {
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
