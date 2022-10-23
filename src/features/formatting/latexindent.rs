use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use lsp_types::{DocumentFormattingParams, TextEdit};
use rowan::{TextLen, TextRange};
use tempfile::tempdir;

use crate::{features::FeatureRequest, DocumentLanguage, LatexindentOptions, LineIndexExt};

pub fn format_with_latexindent(
    request: &FeatureRequest<DocumentFormattingParams>,
) -> Option<Vec<TextEdit>> {
    let document = request.main_document();
    let options = &request.workspace.environment.options;
    let target_dir = tempdir().ok()?;
    let source_dir = options
        .root_directory
        .as_ref()
        .cloned()
        .or_else(|| {
            if document.uri().scheme() == "file" {
                document
                    .uri()
                    .to_file_path()
                    .unwrap()
                    .parent()
                    .map(ToOwned::to_owned)
            } else {
                None
            }
        })
        .unwrap_or_else(|| ".".into());

    let target_file =
        target_dir
            .path()
            .join(if document.data().language() == DocumentLanguage::Bibtex {
                "file.bib"
            } else {
                "file.tex"
            });
    fs::write(&target_file, document.text()).ok()?;

    let args = build_arguments(&options.latexindent, &target_file);

    log::debug!(
        "Running latexindent in folder \"{}\" with args: {:?}",
        source_dir.display(),
        args,
    );

    let output = Command::new("latexindent")
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .current_dir(source_dir)
        .output()
        .ok()?;

    let new_text = String::from_utf8_lossy(&output.stdout).into_owned();
    if new_text.is_empty() {
        None
    } else {
        Some(vec![TextEdit {
            range: document
                .line_index()
                .line_col_lsp_range(TextRange::new(0.into(), document.text().text_len())),
            new_text,
        }])
    }
}

fn build_arguments(options: &LatexindentOptions, target_file: &Path) -> Vec<String> {
    let mut args = Vec::new();

    args.push(match &options.local {
        Some(yaml_file) => format!("--local={yaml_file}"),
        None => "--local".to_string(),
    });

    if options.modify_line_breaks {
        args.push("--modifylinebreaks".to_string());
    }

    args.push(target_file.display().to_string());
    args
}
