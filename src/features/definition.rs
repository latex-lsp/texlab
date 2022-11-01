mod command;
mod document;
mod entry;
mod label;
mod string;

use lsp_types::{GotoDefinitionResponse, LocationLink, Position, Url};
use rowan::TextRange;

use crate::{
    db::{document::Document, workspace::Workspace},
    util::cursor::CursorContext,
    Db, LineIndexExt,
};

pub fn goto_definition(
    db: &dyn Db,
    uri: &Url,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    let document = Workspace::get(db).lookup_uri(db, uri)?;
    let context = CursorContext::new(db, document, position, ());
    log::debug!("[Definition] Cursor: {:?}", context.cursor);

    let origin_document = context.document;
    let links: Vec<_> = command::goto_command_definition(&context)
        .or_else(|| document::goto_document_definition(&context))
        .or_else(|| entry::goto_entry_definition(&context))
        .or_else(|| label::goto_label_definition(&context))
        .or_else(|| string::goto_string_definition(&context))?
        .into_iter()
        .map(|result| {
            let origin_selection_range = Some(
                origin_document
                    .contents(db)
                    .line_index(db)
                    .line_col_lsp_range(result.origin_selection_range),
            );

            let target_line_index = result.target.contents(db).line_index(db);
            let target_uri = result.target.location(context.db).uri(context.db).clone();
            let target_range = target_line_index.line_col_lsp_range(result.target_range);

            let target_selection_range =
                target_line_index.line_col_lsp_range(result.target_selection_range);

            LocationLink {
                origin_selection_range,
                target_uri,
                target_range,
                target_selection_range,
            }
        })
        .collect();

    Some(GotoDefinitionResponse::Link(links))
}

#[derive(Debug, Clone)]
struct DefinitionResult {
    origin_selection_range: TextRange,
    target: Document,
    target_range: TextRange,
    target_selection_range: TextRange,
}
