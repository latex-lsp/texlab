mod command;
mod entry;
mod label;

use lsp_types::{Position, Range, TextEdit, Url, WorkspaceEdit};
use rowan::TextRange;
use rustc_hash::FxHashMap;

use crate::{
    db::Document,
    util::{cursor::CursorContext, line_index_ext::LineIndexExt},
    Db,
};

pub fn prepare_rename_all(db: &dyn Db, uri: &Url, position: Position) -> Option<Range> {
    let context = CursorContext::new(db, uri, position, ())?;
    let range = entry::prepare_rename(&context)
        .or_else(|| label::prepare_rename(&context))
        .or_else(|| command::prepare_rename(&context))?;

    let line_index = context.document.contents(db).line_index(db);
    Some(line_index.line_col_lsp_range(range))
}

pub fn rename_all(
    db: &dyn Db,
    uri: &Url,
    position: Position,
    new_name: String,
) -> Option<WorkspaceEdit> {
    let context = CursorContext::new(db, uri, position, Params { new_name })?;
    let result = entry::rename(&context)
        .or_else(|| label::rename(&context))
        .or_else(|| command::rename(&context))?;

    let changes = result
        .changes
        .into_iter()
        .map(|(document, old_edits)| {
            let line_index = document.contents(db).line_index(db);
            let new_edits = old_edits
                .into_iter()
                .map(|Indel { delete, insert }| {
                    TextEdit::new(line_index.line_col_lsp_range(delete), insert)
                })
                .collect();

            (document.location(db).uri(db).clone(), new_edits)
        })
        .collect();

    Some(WorkspaceEdit::new(changes))
}

#[derive(Debug)]
struct Params {
    new_name: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Indel {
    delete: TextRange,
    insert: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct RenameResult {
    changes: FxHashMap<Document, Vec<Indel>>,
}
