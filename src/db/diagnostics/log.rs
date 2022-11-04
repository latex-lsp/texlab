use lsp_types::{DiagnosticSeverity, Position, Range, Url};
use rustc_hash::FxHashMap;

use crate::{
    db::{document::Document, workspace::Workspace, Distro},
    syntax::BuildErrorLevel,
    Db,
};

use super::{Diagnostic, DiagnosticCode};

#[salsa::tracked(return_ref)]
pub fn collect(
    db: &dyn Db,
    workspace: Workspace,
    distro: Distro,
    log_document: Document,
) -> FxHashMap<Document, Vec<Diagnostic>> {
    let mut results = FxHashMap::default();

    let log = match log_document.parse(db).as_log() {
        Some(data) => data.log(db),
        None => return results,
    };

    let root_document = match workspace
        .documents(db)
        .iter()
        .map(|document| workspace.graph(db, *document, distro))
        .flat_map(|graph| graph.edges(db))
        .find(|edge| edge.target(db) == Some(log_document))
        .map(|edge| edge.source(db))
    {
        Some(document) => document,
        None => return results,
    };

    let base_path = match root_document
        .location(db)
        .path(db)
        .as_deref()
        .and_then(|path| path.parent())
    {
        Some(path) => path,
        None => return results,
    };

    for error in &log.errors {
        let full_path = base_path.join(&error.relative_path);
        let full_path_uri = if let Ok(uri) = Url::from_file_path(&full_path) {
            uri
        } else {
            continue;
        };

        let doc = if error.line.is_some() && error.hint.is_some() {
            workspace.lookup_uri(db, &full_path_uri)
        } else {
            None
        };

        let position: Position = (if let Some(doc) = doc {
            // SAFETY: error.line and error.hint are necessarily Some() if we get here
            let line = error.line.unwrap();
            let hint: &String = error.hint.as_ref().unwrap();
            if let Some(hint_line) = doc.contents(db).text(db).lines().nth(line as usize) {
                hint_line.find(hint).map(|col| {
                    let lc = doc.contents(db).line_index(db).to_utf16(crate::LineCol {
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
        .unwrap_or_else(|| Position::new(error.line.unwrap_or(0), 0));

        let severity = match error.level {
            BuildErrorLevel::Error => DiagnosticSeverity::ERROR,
            BuildErrorLevel::Warning => DiagnosticSeverity::WARNING,
        };
        let range = Range::new(position, position);
        let diagnostic = Diagnostic {
            severity,
            range,
            code: DiagnosticCode::Log(log_document),
            message: error.message.clone(),
        };

        let location = if full_path.starts_with(&base_path) {
            error
                .relative_path
                .to_str()
                .and_then(|path| root_document.location(db).join(db, path))
                .unwrap_or(root_document.location(db))
        } else {
            root_document.location(db)
        };

        if let Some(document) = workspace.lookup(db, location) {
            results.entry(document).or_default().push(diagnostic);
        }
    }

    results
}
