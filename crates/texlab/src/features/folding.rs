use base_db::Workspace;

use crate::util::{ClientFlags, from_proto, to_proto};

pub fn find_all(
    workspace: &Workspace,
    params: lsp_types::FoldingRangeParams,
    client_flags: &ClientFlags,
) -> Option<Vec<serde_json::Value>> {
    let params = from_proto::feature_params(workspace, params.text_document)?;

    let foldings = folding::find_all(params.document)
        .into_iter()
        .filter_map(|folding| {
            to_proto::folding_range(folding, &params.document.line_index, client_flags)
        })
        .collect();

    Some(foldings)
}
