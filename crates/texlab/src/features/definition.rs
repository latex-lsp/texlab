use base_db::Workspace;

use crate::util::{from_proto, to_proto};

pub fn goto_definition(
    workspace: &Workspace,
    params: lsp_types::GotoDefinitionParams,
) -> Option<lsp_types::GotoDefinitionResponse> {
    let params = from_proto::definition_params(workspace, params)?;
    let links = definition::goto_definition(&params)
        .into_iter()
        .filter_map(|result| to_proto::location_link(result, &params.feature.document.line_index))
        .collect();

    Some(lsp_types::GotoDefinitionResponse::Link(links))
}
