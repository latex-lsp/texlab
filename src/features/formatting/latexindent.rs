use std::{
    io::{BufWriter, Write},
    process::{Command, Stdio},
};

use cancellation::CancellationToken;
use cstree::{TextLen, TextRange};
use lsp_types::{DocumentFormattingParams, TextEdit};

use crate::{features::FeatureRequest, LineIndexExt};

pub fn format_with_latexindent(
    request: &FeatureRequest<DocumentFormattingParams>,
    _cancellation_token: &CancellationToken,
) -> Option<Vec<TextEdit>> {
    let document = request.main_document();

    let current_dir = &request.context.current_directory;
    let options = request.context.options.read().unwrap();
    let current_dir = match &options.root_directory {
        Some(root_directory) => current_dir.join(root_directory),
        None => current_dir.clone(),
    };
    drop(options);

    let mut process = Command::new("latexindent")
        .arg("-l")
        .current_dir(current_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let stdin = process.stdin.take()?;
    let mut stdin = BufWriter::new(stdin);
    stdin.write_all(document.text.as_bytes()).ok()?;
    drop(stdin);

    let output = process.wait_with_output().ok()?;

    let new_text = String::from_utf8_lossy(&output.stdout).into_owned();

    Some(vec![TextEdit {
        range: document
            .line_index
            .line_col_lsp_range(TextRange::new(0.into(), document.text.text_len())),
        new_text,
    }])
}
