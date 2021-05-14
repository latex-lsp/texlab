use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    process::{Command, Stdio},
    sync::Arc,
};

use encoding_rs_io::DecodeReaderBytesBuilder;
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range};
use multimap::MultiMap;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{RangeExt, Uri, Workspace};

pub fn analyze_latex_chktex(
    workspace: &dyn Workspace,
    diagnostics_by_uri: &mut MultiMap<Arc<Uri>, Diagnostic>,
    uri: &Uri,
) -> Option<()> {
    let document = workspace.get(uri)?;
    document.data.as_latex()?;
    diagnostics_by_uri.remove(uri);
    diagnostics_by_uri.insert_many(
        Arc::clone(&document.uri),
        lint(&document.text).unwrap_or_default(),
    );
    Some(())
}

pub static LINE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("(\\d+):(\\d+):(\\d+):(\\w+):(\\w+):(.*)").unwrap());

fn lint(text: &str) -> io::Result<Vec<Diagnostic>> {
    let mut process = Command::new("chktex")
        .args(&["-I0", "-f%l:%c:%d:%k:%n:%m\n"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let mut writer = BufWriter::new(process.stdin.take().unwrap());
    writer.write_all(text.as_bytes())?;
    writer.flush()?;
    drop(writer);

    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::UTF_8))
            .utf8_passthru(true)
            .strip_bom(true)
            .build(process.stdout.take().unwrap()),
    );

    let mut diagnostics = Vec::new();
    for line in reader.lines() {
        if let Some(captures) = LINE_REGEX.captures(&line?) {
            let line = captures[1].parse::<u32>().unwrap() - 1;
            let character = captures[2].parse::<u32>().unwrap() - 1;
            let digit = captures[3].parse::<u32>().unwrap();
            let kind = &captures[4];
            let code = &captures[5];
            let message = captures[6].into();
            let range = Range::new_simple(line, character, line, character + digit);
            let severity = match kind {
                "Message" => DiagnosticSeverity::Information,
                "Warning" => DiagnosticSeverity::Warning,
                _ => DiagnosticSeverity::Error,
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
            })
        }
    }

    Ok(diagnostics)
}
