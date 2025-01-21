use std::{
    io::Write,
    process::{Command, Stdio},
};

use base_db::Document;
use rowan::{TextLen, TextRange};

use crate::util::line_index_ext::LineIndexExt;

pub fn format_with_texfmt(document: &Document) -> Option<Vec<lsp_types::TextEdit>> {
    let mut child = Command::new("tex-fmt")
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let mut stdin = child.stdin.take()?;
    std::thread::scope(|s| {
        s.spawn(move || {
            let _ = stdin.write_all(document.text.clone().as_bytes());
        });

        let output = child.wait_with_output().ok()?;
        let new_text = String::from_utf8(output.stdout).ok()?;
        let range = document
            .line_index
            .line_col_lsp_range(TextRange::new(0.into(), document.text.text_len()))?;

        Some(vec![lsp_types::TextEdit { range, new_text }])
    })
}
