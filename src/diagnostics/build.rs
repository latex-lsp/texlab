use std::{path::PathBuf, sync::Arc};

use dashmap::DashMap;
use lsp_types::{DiagnosticSeverity, Position, Range, Url};

use crate::{syntax::build_log::BuildErrorLevel, Workspace};

use super::{Diagnostic, DiagnosticCode};

pub fn collect_build_diagnostics(
    all_diagnostics: &DashMap<Arc<Url>, Vec<Diagnostic>>,
    workspace: &Workspace,
    build_log_uri: &Url,
) -> Option<()> {
    let build_log_document = workspace.documents_by_uri.get(build_log_uri)?;
    let build_log = build_log_document.data.as_build_log()?;

    all_diagnostics.alter_all(|_, mut diagnostics| {
        diagnostics.retain(
            |diag| !matches!(&diag.code, DiagnosticCode::Build(uri) if uri.as_ref() == build_log_uri),
        );
        diagnostics
    });

    let root_document = workspace.documents_by_uri.values().find(|document| {
        document.data.as_latex().map_or(false, |data| {
            !document.uri.as_str().ends_with(".aux")
                && data
                    .extras
                    .implicit_links
                    .log
                    .iter()
                    .any(|u| u.as_ref() == build_log_uri)
        })
    })?;

    let base_path = PathBuf::from(root_document.uri.path());
    for error in &build_log.errors {
        let position = Position::new(error.line.unwrap_or(0), 0);
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

        let full_path = base_path.join(&error.relative_path);

        let uri = if full_path.starts_with(&base_path) {
            error
                .relative_path
                .to_str()
                .and_then(|path| root_document.uri.join(path).map(Into::into).ok())
                .map_or_else(|| Arc::clone(&root_document.uri), Arc::new)
        } else {
            Arc::clone(&root_document.uri)
        };

        all_diagnostics.entry(uri).or_default().push(diagnostic);
    }

    Some(())
}
