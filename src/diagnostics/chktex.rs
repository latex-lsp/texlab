use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
};

use dashmap::DashMap;
use lsp_types::{DiagnosticSeverity, Position, Range, Url};
use once_cell::sync::Lazy;
use regex::Regex;
use tempfile::tempdir;

use crate::{Document, Workspace};

use super::{Diagnostic, DiagnosticCode};

pub fn collect_chktex_diagnostics(
    all_diagnostics: &DashMap<Arc<Url>, Vec<Diagnostic>>,
    workspace: &Workspace,
    uri: &Url,
) -> Option<()> {
    let document = workspace.get(uri)?;
    document.data().as_latex()?;

    all_diagnostics.alter(uri, |_, mut diagnostics| {
        diagnostics.retain(|diag| !matches!(diag.code, DiagnosticCode::Chktex(_)));
        diagnostics
    });

    let current_dir = find_chktexrc_directory(&document)
        .or_else(|| {
            workspace
                .environment
                .options
                .root_directory
                .as_ref()
                .cloned()
        })
        .or_else(|| {
            workspace
                .find_parent(uri)
                .or(Some(document.clone()))
                .filter(|doc| doc.uri().scheme() == "file")
                .and_then(|doc| doc.uri().to_file_path().ok())
                .and_then(|path| path.parent().map(ToOwned::to_owned))
        })
        .unwrap_or_else(|| ".".into());

    log::debug!("Calling ChkTeX from directory: {}", current_dir.display());

    all_diagnostics
        .entry(Arc::clone(document.uri()))
        .or_default()
        .extend(lint(document.text(), &current_dir).unwrap_or_default());

    Some(())
}

static CHKTEXRC_FILES: &[&str] = &["chktexrc", ".chktexrc"];

fn find_chktexrc_directory(document: &Document) -> Option<PathBuf> {
    if document.uri().scheme() == "file" {
        if let Ok(mut path) = document.uri().to_file_path() {
            while path.pop() {
                if CHKTEXRC_FILES
                    .iter()
                    .any(|rc_file| path.join(rc_file).exists())
                {
                    return Some(path);
                }
            }
        }
    }

    None
}

static LINE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("(\\d+):(\\d+):(\\d+):(\\w+):(\\w+):(.*)").unwrap());

fn lint(text: &str, current_dir: &Path) -> io::Result<Vec<Diagnostic>> {
    let directory = tempdir()?;
    fs::write(directory.path().join("file.tex"), text)?;

    for rc_file in CHKTEXRC_FILES {
        let _ = fs::copy(current_dir.join(rc_file), directory.path().join(rc_file));
    }

    let output = Command::new("chktex")
        .args(&["-I0", "-f%l:%c:%d:%k:%n:%m\n", "file.tex"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(directory.path())
        .output()?;

    let mut diagnostics = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let captures = LINE_REGEX.captures(line).unwrap();
        let line = captures[1].parse::<u32>().unwrap() - 1;
        let character = captures[2].parse::<u32>().unwrap() - 1;
        let digit = captures[3].parse::<u32>().unwrap();
        let kind = &captures[4];
        let code = &captures[5];
        let message = captures[6].into();
        let range = Range::new(
            Position::new(line, character),
            Position::new(line, character + digit),
        );
        let severity = match kind {
            "Message" => DiagnosticSeverity::INFORMATION,
            "Warning" => DiagnosticSeverity::WARNING,
            _ => DiagnosticSeverity::ERROR,
        };

        diagnostics.push(Diagnostic {
            range,
            severity,
            code: DiagnosticCode::Chktex(code.into()),
            message,
        });
    }

    Ok(diagnostics)
}
