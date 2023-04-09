mod command;
mod document;
mod entry;
mod label;
mod string;

use base_db::{Document, Workspace};
use lsp_types::{GotoDefinitionResponse, LocationLink, Position, Url};
use rowan::TextRange;

use crate::util::{cursor::CursorContext, line_index_ext::LineIndexExt};

pub fn goto_definition(
    workspace: &Workspace,
    uri: &Url,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    let context = CursorContext::new(workspace, uri, position, ())?;
    log::debug!("[Definition] Cursor: {:?}", context.cursor);

    let links: Vec<_> = command::goto_definition(&context)
        .or_else(|| document::goto_definition(&context))
        .or_else(|| entry::goto_definition(&context))
        .or_else(|| label::goto_definition(&context))
        .or_else(|| string::goto_definition(&context))?
        .into_iter()
        .map(|result| {
            let origin_selection_range = Some(
                context
                    .document
                    .line_index
                    .line_col_lsp_range(result.origin_selection_range),
            );

            let target_line_index = &result.target.line_index;
            let target_uri = result.target.uri.clone();
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
struct DefinitionResult<'a> {
    origin_selection_range: TextRange,
    target: &'a Document,
    target_range: TextRange,
    target_selection_range: TextRange,
}
