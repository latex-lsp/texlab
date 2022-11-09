mod entry;
mod label;
mod string;

use lsp_types::{Location, Position, ReferenceContext, Url};
use rowan::TextRange;

use crate::{
    db::document::Document,
    util::{cursor::CursorContext, line_index_ext::LineIndexExt},
    Db,
};

pub fn find_all(
    db: &dyn Db,
    uri: &Url,
    position: Position,
    params: &ReferenceContext,
) -> Option<Vec<Location>> {
    let mut results = Vec::new();
    let context = CursorContext::new(db, uri, position, params)?;
    log::debug!("[References] Cursor: {:?}", context.cursor);
    label::find_label_references(&context, &mut results);
    entry::find_entry_references(&context, &mut results);
    string::find_string_references(&context, &mut results);

    let locations = results
        .into_iter()
        .map(|result| Location {
            uri: result.document.location(db).uri(db).clone(),
            range: result
                .document
                .contents(db)
                .line_index(db)
                .line_col_lsp_range(result.range),
        })
        .collect();

    Some(locations)
}

#[derive(Debug, Clone)]
struct ReferenceResult {
    document: Document,
    range: TextRange,
}
