use rowan::{TextLen, TextRange, TextSize};
use rustc_hash::FxHashMap;
use syntax::BuildError;
use url::Url;

use crate::{Document, DocumentData, Workspace};

use super::{Diagnostic, ErrorCode};

pub fn analyze(workspace: &Workspace, log_document: &Document) -> FxHashMap<Url, Vec<Diagnostic>> {
    let mut results = FxHashMap::default();

    let DocumentData::Log(data) = &log_document.data else { return results };

    let parents = workspace.parents(log_document);
    let Some(root_document) = parents.iter().next() else { return results };

    let Some(base_path) = root_document.path
        .as_deref()
        .and_then(|path| path.parent()) else { return results };

    for error in &data.errors {
        let full_path = base_path.join(&error.relative_path);
        let Ok(full_path_uri) = Url::from_file_path(&full_path) else { continue };
        let tex_document = workspace.lookup(&full_path_uri).unwrap_or(root_document);

        let range = find_range_of_hint(tex_document, error).unwrap_or_else(|| {
            let line = error.line.unwrap_or(0);
            let offset = *tex_document
                .line_index
                .newlines
                .get(line as usize)
                .unwrap_or(&TextSize::from(0));

            TextRange::empty(offset)
        });

        let diagnostic = Diagnostic {
            range,
            code: ErrorCode::Build(error.clone()),
        };

        results
            .entry(tex_document.uri.clone())
            .or_default()
            .push(diagnostic);
    }

    results
}

fn find_range_of_hint(document: &Document, error: &BuildError) -> Option<TextRange> {
    let line = error.line? as usize;
    let hint = error.hint.as_deref()?;
    let line_index = &document.line_index;

    let line_start = line_index.newlines.get(line).copied()?;
    let line_end = line_index
        .newlines
        .get(line + 1)
        .copied()
        .unwrap_or((&document.text).text_len());

    let line_text = &document.text[line_start.into()..line_end.into()];
    let hint_start = line_start + TextSize::try_from(line_text.find(hint)?).unwrap();
    let hint_end = hint_start + hint.text_len();
    Some(TextRange::new(hint_start, hint_end))
}
