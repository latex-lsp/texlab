use base_db::{DocumentLocation, Workspace};

use crate::util::{ClientFlags, from_proto, to_proto};

pub fn goto_definition(
    workspace: &Workspace,
    params: lsp_types::GotoDefinitionParams,
    client_flags: &ClientFlags,
) -> Option<lsp_types::GotoDefinitionResponse> {
    let params = from_proto::definition_params(workspace, params)?;
    let results = definition::goto_definition(&params);

    if client_flags.location_link_support {
        let links = results
            .into_iter()
            .filter_map(|result| {
                to_proto::location_link(result, &params.feature.document.line_index)
            })
            .collect();

        Some(lsp_types::GotoDefinitionResponse::Link(links))
    } else {
        let locations = results
            .into_iter()
            .map(|result| DocumentLocation::new(result.target, result.target_range))
            .filter_map(to_proto::location)
            .collect();

        Some(lsp_types::GotoDefinitionResponse::Array(locations))
    }
}
