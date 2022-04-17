use std::{
    fs, io,
    path::Path,
    process::{Command, Stdio},
    sync::Arc,
};

use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range};
use multimap::MultiMap;
use once_cell::sync::Lazy;
use regex::Regex;
use tempfile::tempdir;

use crate::{Options, RangeExt, Uri, Workspace};

pub fn analyze_latex_chktex(
    workspace: &dyn Workspace,
    diagnostics_by_uri: &mut MultiMap<Arc<Uri>, Diagnostic>,
    uri: &Uri,
    options: &Options,
) -> Option<()> {
    let document = workspace.get(uri)?;
    document.data.as_latex()?;

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

    diagnostics_by_uri.remove(uri);
    diagnostics_by_uri.insert_many(
        Arc::clone(&document.uri),
        lint(&document.text, &current_dir).unwrap_or_default(),
    );
    Some(())
}

pub static LINE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("(\\d+):(\\d+):(\\d+):(\\w+):(\\w+):(.*)").unwrap());

fn lint(text: &str, current_dir: &Path) -> io::Result<Vec<Diagnostic>> {
    let directory = tempdir()?;
    fs::write(directory.path().join("file.tex"), text)?;
    let _ = fs::copy(
        current_dir.join("chktexrc"),
        directory.path().join("chktexrc"),
    );

    let output = Command::new("chktex")
        .args(&["-I0", "-f%l:%c:%d:%k:%n:%m\n", "file.tex"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(directory.path())
        .output()?;

    // let mut writer = BufWriter::new(process.stdin.take().unwrap());
    // writer.write_all(text.as_bytes())?;
    // // writer.flush()?;
    // let output = process.wait_with_output()?;

    let mut diagnostics = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let captures = LINE_REGEX.captures(line).unwrap();
        let line = captures[1].parse::<u32>().unwrap() - 1;
        let character = captures[2].parse::<u32>().unwrap() - 1;
        let digit = captures[3].parse::<u32>().unwrap();
        let kind = &captures[4];
        let code = &captures[5];
        let message = captures[6].into();
        let range = Range::new_simple(line, character, line, character + digit);
        let severity = match kind {
            "Message" => DiagnosticSeverity::INFORMATION,
            "Warning" => DiagnosticSeverity::WARNING,
            _ => DiagnosticSeverity::ERROR,
        };

        diagnostics.push(Diagnostic {
            range,
            severity: Some(severity),
            code: Some(NumberOrString::String(code.into())),
            code_description: None,
            source: Some("chktex".into()),
            message,
            related_information: None,
            tags: None,
            data: None,
        });
    }

    Ok(diagnostics)
}
