mod entry;
mod label;
mod string;

use base_db::{Document, Workspace};
use lsp_types::{Location, Position, ReferenceContext, Url};
use rowan::TextRange;

use crate::util::{cursor::CursorContext, line_index_ext::LineIndexExt};

pub fn find_all(
    workspace: &Workspace,
    uri: &Url,
    position: Position,
    params: &ReferenceContext,
) -> Option<Vec<Location>> {
    let mut results = Vec::new();
    let context = CursorContext::new(workspace, uri, position, params)?;
    log::debug!("[References] Cursor: {:?}", context.cursor);
    label::find_all_references(&context, &mut results);
    entry::find_all_references(&context, &mut results);
    string::find_all_references(&context, &mut results);

    let locations = results
        .into_iter()
        .map(|result| Location {
            uri: result.document.uri.clone(),
            range: result.document.line_index.line_col_lsp_range(result.range),
        })
        .collect();

    Some(locations)
}

#[derive(Debug)]
struct ReferenceResult<'a> {
    document: &'a Document,
    range: TextRange,
}
