use lsp_types::{DiagnosticSeverity, Position, Range, Url};
use rowan::{TextLen, TextRange, TextSize};
use rustc_hash::FxHashMap;
use syntax::{BuildError, BuildErrorLevel};

use crate::{
    db::{document::Document, workspace::Workspace},
    util::line_index_ext::LineIndexExt,
    Db,
};

use super::{Diagnostic, DiagnosticCode};

#[salsa::tracked(return_ref)]
pub fn collect(
    db: &dyn Db,
    workspace: Workspace,
    log_document: Document,
) -> FxHashMap<Document, Vec<Diagnostic>> {
    let mut results = FxHashMap::default();

    let log = match log_document.parse(db).as_log() {
        Some(data) => data.log(db),
        None => return results,
    };

    let root_document = match workspace.parents(db, log_document).iter().next().copied() {
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

        let severity = match error.level {
            BuildErrorLevel::Error => DiagnosticSeverity::ERROR,
            BuildErrorLevel::Warning => DiagnosticSeverity::WARNING,
        };

        let range = find_range_of_hint(db, workspace, &full_path_uri, error).unwrap_or_else(|| {
            let line = error.line.unwrap_or(0);
            Range::new(Position::new(line, 0), Position::new(line, 0))
        });

        let diagnostic = Diagnostic {
            severity,
            range,
            code: DiagnosticCode::Log(log_document),
            message: error.message.clone(),
        };

        let tex_document = workspace
            .lookup_uri(db, &full_path_uri)
            .unwrap_or(root_document);

        results.entry(tex_document).or_default().push(diagnostic);
    }

    results
}

fn find_range_of_hint(
    db: &dyn Db,
    workspace: Workspace,
    uri: &Url,
    error: &BuildError,
) -> Option<Range> {
    let document = workspace.lookup_uri(db, uri)?;
    let text = document.text(db);
    let line = error.line? as usize;
    let hint = error.hint.as_deref()?;
    let line_index = document.line_index(db);

    let line_start = line_index.newlines.get(line).copied()?;
    let line_end = line_index
        .newlines
        .get(line + 1)
        .copied()
        .unwrap_or(text.text_len());

    let line_text = &text[line_start.into()..line_end.into()];
    let hint_start = line_start + TextSize::try_from(line_text.find(hint)?).unwrap();
    let hint_end = hint_start + hint.text_len();
    Some(line_index.line_col_lsp_range(TextRange::new(hint_start, hint_end)))
}
