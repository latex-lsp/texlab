use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::Stdio,
};

use base_db::{
    deps::{self, ProjectRoot},
    Document, Workspace,
};
use encoding_rs_io::DecodeReaderBytesBuilder;
use line_index::LineCol;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{types::Diagnostic, ChktexError, ChktexSeverity};

#[derive(Debug)]
pub struct Command {
    text: String,
    working_dir: PathBuf,
    additional_args: Vec<String>,
}

impl Command {
    pub fn new(workspace: &Workspace, document: &Document) -> Option<Self> {
        document.data.as_tex()?;

        let parent = deps::parents(workspace, document)
            .into_iter()
            .next()
            .unwrap_or(document);

        if parent.path.is_none() {
            log::warn!("Calling ChkTeX on non-local files is not supported yet.");
            return None;
        }

        let root = ProjectRoot::walk_and_find(workspace, parent.dir.as_ref()?);

        let working_dir = root.src_dir.to_file_path().ok()?;
        log::debug!("Calling ChkTeX from directory: {}", working_dir.display());

        let text = document.text.clone();
        let config = &workspace.config().diagnostics.chktex;
        let additional_args = config.additional_args.clone();
        Some(Self {
            text,
            working_dir,
            additional_args,
        })
    }

    pub fn run(mut self) -> std::io::Result<Vec<Diagnostic>> {
        let mut args = vec!["-I0".into(), "-f%l:%c:%d:%k:%n:%m\n".into()];
        args.append(&mut self.additional_args);

        let mut child = std::process::Command::new("chktex")
            .args(args)
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

            for line in reader.lines().map_while(Result::ok) {
                let captures = LINE_REGEX.captures(&line).unwrap();
                let line = captures[1].parse::<u32>().unwrap() - 1;
                let character = captures[2].parse::<u32>().unwrap() - 1;
                let digit = captures[3].parse::<u32>().unwrap();
                let kind = &captures[4];
                let code = String::from(&captures[5]);
                let message = captures[6].into();
                let start = LineCol {
                    line,
                    col: character,
                };

                let end = LineCol {
                    line,
                    col: character + digit,
                };

                let severity = match kind {
                    "Message" => ChktexSeverity::Message,
                    "Warning" => ChktexSeverity::Warning,
                    _ => ChktexSeverity::Error,
                };

                diagnostics.push(Diagnostic::Chktex(ChktexError {
                    start,
                    end,
                    message,
                    severity,
                    code,
                }));
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
