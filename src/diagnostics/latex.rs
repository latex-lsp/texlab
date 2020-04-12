use crate::workspace::Document;
use chashmap::CHashMap;
use futures::{
    future::{AbortHandle, Abortable, Aborted},
    lock::Mutex,
};
use log::trace;
use once_cell::sync::Lazy;
use regex::Regex;
use std::process::Stdio;
use texlab_protocol::{Diagnostic, DiagnosticSeverity, NumberOrString, Range, RangeExt, Uri};
use tokio::{prelude::*, process::Command};

#[derive(Debug, Default)]
pub struct LatexDiagnosticsProvider {
    diagnostics_by_uri: CHashMap<Uri, Vec<Diagnostic>>,
    handle: Mutex<Option<AbortHandle>>,
}

impl LatexDiagnosticsProvider {
    pub fn get(&self, document: &Document) -> Vec<Diagnostic> {
        match self.diagnostics_by_uri.get(&document.uri) {
            Some(diagnostics) => diagnostics.to_owned(),
            None => Vec::new(),
        }
    }

    pub async fn update(&self, uri: &Uri, text: &str) {
        if uri.scheme() != "file" {
            return;
        }

        let mut handle_guard = self.handle.lock().await;
        if let Some(handle) = &*handle_guard {
            handle.abort();
        }

        let (handle, registration) = AbortHandle::new_pair();
        *handle_guard = Some(handle);
        drop(handle_guard);

        let future = Abortable::new(
            async move {
                self.diagnostics_by_uri
                    .insert(uri.clone(), lint(text.into()).await.unwrap_or_default());
            },
            registration,
        );

        if let Err(Aborted) = future.await {
            trace!("Killed ChkTeX because it took too long to execute")
        }
    }
}

pub static LINE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("(\\d+):(\\d+):(\\d+):(\\w+):(\\w+):(.*)").unwrap());

async fn lint(text: String) -> io::Result<Vec<Diagnostic>> {
    let mut process: tokio::process::Child = Command::new("chktex")
        .args(&["-I0", "-f%l:%c:%d:%k:%n:%m\n"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;

    process
        .stdin
        .take()
        .unwrap()
        .write_all(text.as_bytes())
        .await?;

    let mut stdout = String::new();
    process
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut stdout)
        .await?;

    let mut diagnostics = Vec::new();
    for line in stdout.lines() {
        if let Some(captures) = LINE_REGEX.captures(line) {
            let line = captures[1].parse::<u64>().unwrap() - 1;
            let character = captures[2].parse::<u64>().unwrap() - 1;
            let digit = captures[3].parse::<u64>().unwrap();
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
                source: Some("chktex".into()),
                code: Some(NumberOrString::String(code.into())),
                message,
                severity: Some(severity),
                range,
                related_information: None,
                tags: None,
            })
        }
    }
    Ok(diagnostics)
}
