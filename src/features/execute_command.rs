use std::{path::PathBuf, process::Stdio, sync::Arc};

use anyhow::Result;
use lsp_types::{TextDocumentIdentifier, Url};

use crate::Workspace;

pub fn execute_command(
    workspace: &Workspace,
    name: &str,
    args: Vec<serde_json::Value>,
) -> Result<()> {
    match name {
        "texlab.cleanAuxiliary" => {
            let params = args
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("texlab.cleanAuxiliary requires one argument"))?;

            clean_output_files(workspace, CleanOptions::Auxiliary, params)?;
        }
        "texlab.cleanArtifacts" => {
            let params = args
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("texlab.cleanArtifacts requires one argument"))?;

            clean_output_files(workspace, CleanOptions::Artifacts, params)?;
        }
        _ => anyhow::bail!("Unknown command: {}", name),
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
enum CleanOptions {
    Auxiliary,
    Artifacts,
}

fn clean_output_files(
    workspace: &Workspace,
    options: CleanOptions,
    params: serde_json::Value,
) -> Result<()> {
    let params: TextDocumentIdentifier = serde_json::from_value(params)?;

    let uri = workspace
        .find_parent(&params.uri)
        .map(|document| document.uri)
        .unwrap_or_else(|| Arc::new(params.uri));

    if let Some(cx) = BuildContext::find(workspace, &uri) {
        let flag = match options {
            CleanOptions::Auxiliary => "-c",
            CleanOptions::Artifacts => "-C",
        };

        std::process::Command::new("latexmk")
            .arg(format!("-outdir={}", cx.output_dir.to_string_lossy()))
            .arg(flag)
            .arg(cx.input_file)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
    }

    Ok(())
}

struct BuildContext {
    input_file: PathBuf,
    output_dir: PathBuf,
}

impl BuildContext {
    pub fn find(workspace: &Workspace, uri: &Url) -> Option<Self> {
        if uri.scheme() != "file" {
            return None;
        }

        let input_file = uri.to_file_path().ok()?;
        let options = &workspace.environment.options;
        let current_dir = &workspace.environment.current_directory;
        let output_dir = match (
            options.root_directory.as_ref(),
            options.aux_directory.as_ref(),
        ) {
            (_, Some(aux_dir)) => current_dir.join(aux_dir),
            (Some(root_dir), None) => current_dir.join(root_dir),
            (None, None) => input_file.parent()?.to_path_buf(),
        };

        log::info!("Output = {:#?}", output_dir);

        Some(Self {
            input_file,
            output_dir,
        })
    }
}
