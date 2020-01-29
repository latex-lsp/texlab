use log::*;
use std::io;
use std::path::Path;
use std::process::Stdio;
use texlab_protocol::*;
use tokio::process::Command;

pub async fn search<'a>(
    tex_file: &'a Path,
    parent: &'a Path,
    line_number: u64,
    options: Options,
) -> Option<ForwardSearchResult> {
    let pdf_file = options.resolve_output_file(parent, "pdf").unwrap();

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

    let args: Vec<String> = search_options
        .args
        .unwrap()
        .into_iter()
        .flat_map(|arg| replace_placeholder(&tex_file, &pdf_file, line_number, arg))
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
