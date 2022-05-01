mod command;
mod document;
mod entry;
mod label;
mod string;

use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse};

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
    let links = goto_command_definition(&context)
        .or_else(|| goto_document_definition(&context))
        .or_else(|| goto_entry_definition(&context))
        .or_else(|| goto_label_definition(&context))
        .or_else(|| goto_string_definition(&context))?;
    Some(GotoDefinitionResponse::Link(links))
}
