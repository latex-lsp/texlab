mod command;
mod entry;
mod label;
mod string;

use cancellation::CancellationToken;
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse};

use self::{
    command::goto_command_definition, entry::goto_entry_definition, label::goto_label_definition,
    string::goto_string_definition,
};

use super::FeatureRequest;

pub fn goto_definition(
    request: FeatureRequest<GotoDefinitionParams>,
    token: &CancellationToken,
) -> Option<GotoDefinitionResponse> {
    let links = goto_command_definition(&request, token)
        .or_else(|| goto_entry_definition(&request, token))
        .or_else(|| goto_label_definition(&request, token))
        .or_else(|| goto_string_definition(&request, token))?;
    Some(GotoDefinitionResponse::Link(links))
}
