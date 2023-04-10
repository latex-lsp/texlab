use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::Stdio,
};

use base_db::{Document, Workspace};
use distro::Language;
use encoding_rs_io::DecodeReaderBytesBuilder;
use lsp_types::{Diagnostic, NumberOrString};
use lsp_types::{DiagnosticSeverity, Position, Range};
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub struct Command {
    text: String,
    working_dir: PathBuf,
}

impl Command {
    pub fn new(workspace: &Workspace, document: &Document) -> Option<Self> {
        if document.language != Language::Tex {
            return None;
        }

        let parent = workspace
            .parents(document)
            .into_iter()
            .next()
            .unwrap_or(document);

        if parent.uri.scheme() != "file" {
            log::warn!("Calling ChkTeX on non-local files is not supported yet.");
            return None;
        }

        let working_dir = workspace.current_dir(&parent.dir).to_file_path().ok()?;
        log::debug!("Calling ChkTeX from directory: {}", working_dir.display());

        let text = document.text.clone();

        Some(Self { text, working_dir })
    }

    pub fn run(self) -> std::io::Result<Vec<Diagnostic>> {
        let mut child = std::process::Command::new("chktex")
            .args(["-I0", "-f%l:%c:%d:%k:%n:%m\n"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .current_dir(self.working_dir)
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let reader = std::thread::spawn(move || {
            let mut diagnostics = Vec::new();
            let reader = BufReader::new(
                DecodeReaderBytesBuilder::new()
                    .encoding(Some(encoding_rs::UTF_8))
                    .utf8_passthru(true)
                    .strip_bom(true)
                    .build(stdout),
            );

            for line in reader.lines().flatten() {
                let captures = LINE_REGEX.captures(&line).unwrap();
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
                    severity: Some(severity),
                    code: Some(NumberOrString::String(code.into())),
                    message,
                    code_description: None,
                    source: Some(String::from("ChkTeX")),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }

            diagnostics
        });

        let mut stdin = child.stdin.take().unwrap();
        let bytes = self.text.into_bytes();
        let writer = std::thread::spawn(move || stdin.write_all(&bytes));

        child.wait()?;
        writer.join().unwrap()?;
        let diagnostics = reader.join().unwrap();
        Ok(diagnostics)
    }
}

static LINE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("(\\d+):(\\d+):(\\d+):(\\w+):(\\w+):(.*)").unwrap());
