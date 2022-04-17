use std::{path::PathBuf, sync::Arc};

use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use multimap::MultiMap;

use crate::{syntax::build_log::BuildErrorLevel, Uri, Workspace};

pub fn analyze_build_log_static(
    workspace: &Workspace,
    diagnostics_by_uri: &mut MultiMap<Arc<Uri>, Diagnostic>,
    build_log_uri: &Uri,
) -> Option<()> {
    let build_log_document = workspace.documents_by_uri.get(build_log_uri)?;
    let parse = build_log_document.data.as_build_log()?;

    let root_document = workspace.documents_by_uri.values().find(|document| {
        if let Some(data) = document.data.as_latex() {
            !document.uri.as_str().ends_with(".aux")
                && data
                    .extras
                    .implicit_links
                    .log
                    .iter()
                    .any(|u| u.as_ref() == build_log_uri)
        } else {
            false
        }
    })?;

    let base_path = PathBuf::from(root_document.uri.path());

    for error in &parse.errors {
        let pos = Position::new(error.line.unwrap_or(0), 0);
        let severity = match error.level {
            BuildErrorLevel::Error => DiagnosticSeverity::ERROR,
            BuildErrorLevel::Warning => DiagnosticSeverity::WARNING,
        };
        let range = Range::new(pos, pos);
        let diagnostic = Diagnostic {
            range,
            severity: Some(severity),
            code: None,
            code_description: None,
            source: Some("latex".into()),
            message: error.message.clone(),
            related_information: None,
            tags: None,
            data: None,
        };

        let full_path = base_path.join(&error.relative_path);

        let uri = if full_path.starts_with(&base_path) {
            error
                .relative_path
                .to_str()
                .and_then(|path| root_document.uri.join(path).map(Into::into).ok())
                .map(Arc::new)
                .unwrap_or_else(|| Arc::clone(&root_document.uri))
        } else {
            Arc::clone(&root_document.uri)
        };

        diagnostics_by_uri.insert(uri, diagnostic);
    }
    Some(())
}
