use std::{
    path::Path,
    process::{Command, Stdio},
};

use base_db::{deps::ProjectRoot, Document, LatexIndentConfig, Workspace};
use distro::Language;
use rowan::TextLen;
use tempfile::tempdir;

use crate::util::line_index_ext::LineIndexExt;

pub fn format_with_latexindent(
    workspace: &Workspace,
    document: &Document,
) -> Option<Vec<lsp_types::TextEdit>> {
    let config = workspace.config();
    let target_dir = tempdir().ok()?;
    let root = ProjectRoot::walk_and_find(workspace, document.dir.as_ref()?);
    let source_dir = root.src_dir.to_file_path().ok()?;

    let target_file = target_dir
        .path()
        .join(if document.language == Language::Bib {
            "file.bib"
        } else {
            "file.tex"
        });
    std::fs::write(&target_file, &document.text).ok()?;

    let args = build_arguments(&config.formatting.latex_indent, &target_file);

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

    let old_text = &document.text;
    let new_text = String::from_utf8_lossy(&output.stdout).into_owned();
    if new_text.is_empty() {
        None
    } else {
        let line_index = &document.line_index;
        let start = lsp_types::Position::new(0, 0);
        let end = line_index.line_col_lsp(old_text.text_len())?;
        Some(vec![lsp_types::TextEdit {
            range: lsp_types::Range::new(start, end),
            new_text,
        }])
    }
}

fn build_arguments(config: &LatexIndentConfig, target_file: &Path) -> Vec<String> {
    let mut args = Vec::new();

    args.push(match &config.local {
        Some(yaml_file) => format!("--local={yaml_file}"),
        None => "--local".to_string(),
    });

    if config.modify_line_breaks {
        args.push("--modifylinebreaks".to_string());
    }

    match &config.replacement {
        Some(replacement_flag) => {
            if ["-r", "-rv", "-rr"].contains(&replacement_flag.as_str()) {
                args.push(replacement_flag.clone());
            }
        }
        None => {}
    }

    args.push(target_file.display().to_string());
    args
}
