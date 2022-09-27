use std::sync::Arc;

use dashmap::DashMap;
use lsp_types::{DiagnosticSeverity, Position, Range, Url};

use crate::{syntax::BuildErrorLevel, Workspace};

use super::{Diagnostic, DiagnosticCode};

pub fn collect_build_diagnostics(
    all_diagnostics: &DashMap<Arc<Url>, Vec<Diagnostic>>,
    workspace: &Workspace,
    build_log_uri: &Url,
) -> Option<()> {
    let build_log_document = workspace.documents_by_uri.get(build_log_uri)?;
    let build_log = build_log_document.data.as_build_log()?.to_owned();

    all_diagnostics.alter_all(|_, mut diagnostics| {
        diagnostics.retain(
            |diag| !matches!(&diag.code, DiagnosticCode::Build(uri) if uri.as_ref() == build_log_uri),
        );
        diagnostics
    });

    let root_document_uri = &workspace
        .documents_by_uri
        .values()
        .find(|document| {
            document.data.as_latex().map_or(false, |data| {
                !document.uri.as_str().ends_with(".aux")
                    && data
                        .extras
                        .implicit_links
                        .log
                        .iter()
                        .any(|u| u.as_ref() == build_log_uri)
            })
        })?
        .uri;

    let mut base_path = root_document_uri.to_file_path().ok()?;
    base_path.pop();
    for error in &build_log.errors {
        let full_path = base_path.join(&error.relative_path);
        let full_path_uri = if let Ok(uri) = Url::from_file_path(&full_path) {
            uri
        } else {
            continue;
        };

        let doc = if error.line.is_some() && error.hint.is_some() {
            workspace.documents_by_uri.get(&full_path_uri)
        } else {
            None
        };

        let position: Position = (if let Some(doc) = doc {
            // SAFETY: error.line and error.hint are necessarily Some() if we get here
            let line = error.line.unwrap();
            let hint: &String = error.hint.as_ref().unwrap();
            if let Some(hint_line) = doc.text.lines().nth(line as usize) {
                hint_line.find(hint).map(|col| {
                    let lc = doc.line_index.to_utf16(crate::LineCol {
                        line,
                        col: (col + hint.len() - 1) as u32,
                    });
                    Position::new(lc.line, lc.col)
                })
            } else {
                log::warn!(
                    "Invalid line number {} in \"{}\" for \"{}\"",
                    line,
                    full_path.display(),
                    error.message
                );
                None
            }
        } else {
            None
        })
        .unwrap_or(Position::new(error.line.unwrap_or(0), 0));

        let severity = match error.level {
            BuildErrorLevel::Error => DiagnosticSeverity::ERROR,
            BuildErrorLevel::Warning => DiagnosticSeverity::WARNING,
        };
        let range = Range::new(position, position);
        let diagnostic = Diagnostic {
            severity,
            range,
            code: DiagnosticCode::Build(Arc::clone(&build_log_document.uri)),
            message: error.message.clone(),
        };

        let uri = if full_path.starts_with(&base_path) {
            error
                .relative_path
                .to_str()
                .and_then(|path| root_document_uri.join(path).map(Into::into).ok())
                .map_or_else(|| Arc::clone(root_document_uri), Arc::new)
        } else {
            Arc::clone(root_document_uri)
        };

        all_diagnostics.entry(uri).or_default().push(diagnostic);
    }

    Some(())
}
