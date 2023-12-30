use base_db::{Document, Workspace};
use line_index::LineCol;
use multimap::MultiMap;
use rowan::{TextLen, TextRange, TextSize};
use rustc_hash::FxHashMap;
use syntax::BuildError;
use url::Url;

use crate::types::Diagnostic;

pub fn update(
    workspace: &Workspace,
    log_document: &Document,
    results: &mut FxHashMap<Url, MultiMap<Url, Diagnostic>>,
) -> Option<()> {
    let mut errors = MultiMap::default();

    let data = log_document.data.as_log()?;

    let parents = workspace.parents(log_document);
    let root_document = parents.iter().next()?;

    let base_path = root_document
        .path
        .as_deref()
        .and_then(|path| path.parent())?;

    for error in &data.errors {
        let full_path = base_path.join(&error.relative_path);
        let Ok(full_path_uri) = Url::from_file_path(&full_path) else {
            continue;
        };

        let tex_document = workspace.lookup(&full_path_uri).unwrap_or(root_document);

        let range = find_range_of_hint(tex_document, error).unwrap_or_else(|| {
            let line = error.line.unwrap_or(0);
            let offset = tex_document
                .line_index
                .offset(LineCol { line, col: 0 })
                .unwrap_or(TextSize::from(0));

            TextRange::empty(offset)
        });

        let diagnostic = Diagnostic::Build(range, error.clone());
        errors.insert(tex_document.uri.clone(), diagnostic);
    }

    results.insert(log_document.uri.clone(), errors);
    Some(())
}

fn find_range_of_hint(document: &Document, error: &BuildError) -> Option<TextRange> {
    let line = error.line?;
    let hint = error.hint.as_deref()?;
    let line_index = &document.line_index;

    let line_start = line_index.offset(LineCol { line, col: 0 })?;
    let line_end = line_index
        .offset(LineCol {
            line: line + 1,
            col: 0,
        })
        .unwrap_or_else(|| document.text.text_len());

    let line_text = &document.text[line_start.into()..line_end.into()];
    let hint_start = line_start + TextSize::try_from(line_text.find(hint)?).unwrap();
    let hint_end = hint_start + hint.text_len();
    Some(TextRange::new(hint_start, hint_end))
}
