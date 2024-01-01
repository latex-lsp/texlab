use base_db::Workspace;

use crate::util::{from_proto, to_proto};

pub fn find_all(
    workspace: &Workspace,
    params: lsp_types::ReferenceParams,
) -> Option<Vec<lsp_types::Location>> {
    let params = from_proto::reference_params(workspace, params)?;

    let results = references::find_all(&params)
        .into_iter()
        .filter_map(to_proto::location)
        .collect();

    Some(results)
}
