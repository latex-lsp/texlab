use std::{
    fs,
    process::{Command, Stdio},
};

use cancellation::CancellationToken;
use lsp_types::{DocumentFormattingParams, TextEdit};
use rowan::{TextLen, TextRange};
use tempfile::tempdir;

use crate::{features::FeatureRequest, DocumentLanguage, LineIndexExt};

pub fn format_with_latexindent(
    request: &FeatureRequest<DocumentFormattingParams>,
    _cancellation_token: &CancellationToken,
) -> Option<Vec<TextEdit>> {
    let directory = tempdir().ok()?;
    let document = request.main_document();

    let options = request.context.options.read().unwrap();
    let current_dir = options
        .root_directory
        .as_ref()
        .cloned()
        .or_else(|| {
            if document.uri.scheme() == "file" {
                document
                    .uri
                    .to_file_path()
                    .unwrap()
                    .parent()
                    .map(ToOwned::to_owned)
            } else {
                None
            }
        })
        .unwrap_or_else(|| ".".into());

    let local = match &options.latexindent.local {
        Some(local) => format!("--local={}", local),
        None => "-l".to_string(),
    };

    let modify_line_breaks = options.latexindent.modify_line_breaks;

    drop(options);

    let path = directory.path();
    let _ = fs::copy(
        current_dir.join("localSettings.yaml"),
        path.join("localSettings.yaml"),
    );
    let _ = fs::copy(
        current_dir.join(".localSettings.yaml"),
        path.join(".localSettings.yaml"),
    );
    let _ = fs::copy(
        current_dir.join("latexindent.yaml"),
        path.join("latexindent.yaml"),
    );

    let name = if document.language() == DocumentLanguage::Bibtex {
        "file.bib"
    } else {
        "file.tex"
    };

    fs::write(directory.path().join(name), &document.text).ok()?;

    let mut args = Vec::new();
    if modify_line_breaks {
        args.push("--modifylinebreaks");
    }
    args.push(&local);
    args.push(name);

    let output = Command::new("latexindent")
        .args(&args)
        .current_dir(current_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .current_dir(directory.path())
        .output()
        .ok()?;

    let new_text = String::from_utf8_lossy(&output.stdout).into_owned();
    if new_text.is_empty() {
        None
    } else {
        Some(vec![TextEdit {
            range: document
                .line_index
                .line_col_lsp_range(TextRange::new(0.into(), document.text.text_len())),
            new_text,
        }])
    }
}
