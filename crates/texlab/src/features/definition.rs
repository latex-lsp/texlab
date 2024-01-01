use base_db::Workspace;
use lsp_types::GotoDefinitionResponse;

use crate::util::{from_proto, to_proto};

pub fn goto_definition(
    workspace: &Workspace,
    params: lsp_types::GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let params = from_proto::definition_params(workspace, params)?;
    let links = definition::goto_definition(&params)
        .into_iter()
        .filter_map(|result| to_proto::location_link(result, &params.feature.document.line_index))
        .collect();

    Some(GotoDefinitionResponse::Link(links))
}
