use crate::workspace::Document;
use lazy_static::lazy_static;
use lsp_types::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LatexLintOptions {
    pub on_save: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LatexDiagnosticsProvider {
    diagnostics_by_uri: HashMap<Uri, Vec<Diagnostic>>,
}

impl LatexDiagnosticsProvider {
    pub fn get(&self, document: &Document) -> Vec<Diagnostic> {
        match self.diagnostics_by_uri.get(&document.uri) {
            Some(diagnostics) => diagnostics.to_owned(),
            None => Vec::new(),
        }
    }

    pub fn update(&mut self, uri: &Uri) {
        if uri.scheme() != "file" {
            return;
        }

        let path = uri.to_file_path().unwrap();
        self.diagnostics_by_uri
            .insert(uri.clone(), lint(&path).unwrap_or_default());
    }
}

const LINE_PATTERN: &str = "(\\d+):(\\d+):(\\d+):(\\w+):(\\w)+:(.*)";

lazy_static! {
    static ref LINE_REGEX: Regex = Regex::new(LINE_PATTERN).unwrap();
}

fn lint(path: &Path) -> Option<Vec<Diagnostic>> {
    let file = File::open(path).ok()?;
    let output = Command::new("chktex")
        .args(&["-I0", "-f%l:%c:%d:%k:%n:%m\n"])
        .stdin(file)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let mut diagnostics = Vec::new();
    let stdout = String::from_utf8(output.stdout).ok()?;
    for line in stdout.lines() {
        if let Some(captures) = LINE_REGEX.captures(line) {
            let line = captures[1].parse::<u64>().unwrap() - 1;
            let character = captures[2].parse::<u64>().unwrap() - 1;
            let digit = captures[3].parse::<u64>().unwrap();
            let kind = &captures[4];
            let code = &captures[5];
            let message = captures[6].to_owned();
            let range = Range::new_simple(line, character, line, character + digit);
            let severity = match kind {
                "Message" => DiagnosticSeverity::Information,
                "Warning" => DiagnosticSeverity::Warning,
                _ => DiagnosticSeverity::Error,
            };

            diagnostics.push(Diagnostic {
                source: Some(Cow::from("chktex")),
                code: Some(NumberOrString::String(code.to_owned())),
                message: Cow::from(message),
                severity: Some(severity),
                range,
                related_information: None,
            })
        }
    }
    Some(diagnostics)
}
