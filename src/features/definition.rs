mod command;
mod document;
mod entry;
mod label;
mod string;

use std::sync::Arc;

use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, LocationLink, Url};
use rowan::TextRange;

use crate::LineIndexExt;

use self::{
    command::goto_command_definition, document::goto_document_definition,
    entry::goto_entry_definition, label::goto_label_definition, string::goto_string_definition,
};

use super::{cursor::CursorContext, FeatureRequest};

pub fn goto_definition(
    request: FeatureRequest<GotoDefinitionParams>,
) -> Option<GotoDefinitionResponse> {
    let context = CursorContext::new(request);
    log::debug!("[Definition] Cursor: {:?}", context.cursor);

    let origin_document = context.request.main_document();
    let links: Vec<_> = goto_command_definition(&context)
        .or_else(|| goto_document_definition(&context))
        .or_else(|| goto_entry_definition(&context))
        .or_else(|| goto_label_definition(&context))
        .or_else(|| goto_string_definition(&context))?
        .into_iter()
        .map(|result| {
            let origin_selection_range = Some(
                origin_document
                    .line_index
                    .line_col_lsp_range(result.origin_selection_range),
            );

            let target_document = &context.request.workspace.documents_by_uri[&result.target_uri];
            let target_uri = result.target_uri.as_ref().clone();
            let target_range = target_document
                .line_index
                .line_col_lsp_range(result.target_range);
            let target_selection_range = target_document
                .line_index
                .line_col_lsp_range(result.target_selection_range);

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
    target_uri: Arc<Url>,
    target_range: TextRange,
    target_selection_range: TextRange,
}

#[cfg(test)]
mod tests;
