use base_db::Workspace;

use crate::util::{to_proto, ClientFlags};

pub fn find_all(
    workspace: &Workspace,
    uri: &lsp_types::Url,
    client_flags: &ClientFlags,
) -> Option<Vec<serde_json::Value>> {
    let document = workspace.lookup(uri)?;
    let foldings = folding::find_all(document)
        .into_iter()
        .filter_map(|folding| to_proto::folding_range(folding, &document.line_index, client_flags))
        .collect();

    Some(foldings)
}
