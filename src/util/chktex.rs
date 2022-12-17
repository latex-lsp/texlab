use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::Stdio,
};

use encoding_rs_io::DecodeReaderBytesBuilder;
use lsp_types::{DiagnosticSeverity, Position, Range};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    db::{
        diagnostics::{Diagnostic, DiagnosticCode},
        document::Document,
        workspace::Workspace,
    },
    Db,
};

#[derive(Debug)]
pub struct Command {
    text: String,
    working_dir: PathBuf,
}

impl Command {
    pub fn new(db: &dyn Db, document: Document) -> Option<Self> {
        document.parse(db).as_tex()?;

        let workspace = Workspace::get(db);
        let parent = workspace
            .parents(db, document)
            .iter()
            .next()
            .map_or(document, Clone::clone);

        let working_dir = workspace
            .working_dir(db, parent.directory(db))
            .path(db)
            .as_deref()?
            .to_owned();

        log::debug!("Calling ChkTeX from directory: {}", working_dir.display());

        let text = document.contents(db).text(db).clone();

        Some(Self { text, working_dir })
    }

    pub fn run(self) -> std::io::Result<Vec<Diagnostic>> {
        let mut child = std::process::Command::new("chktex")
            .args(&["-I0", "-f%l:%c:%d:%k:%n:%m\n"])
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
                    severity,
                    code: DiagnosticCode::Chktex(code.into()),
                    message,
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
