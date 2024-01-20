use base_db::Workspace;

use crate::util::{from_proto, to_proto};

pub fn find_all(
    workspace: &Workspace,
    params: lsp_types::DocumentLinkParams,
) -> Option<Vec<lsp_types::DocumentLink>> {
    let params = from_proto::feature_params(workspace, params.text_document)?;
    let links = links::find_links(&params)
        .into_iter()
        .filter_map(|link| to_proto::document_link(link, &params.document.line_index))
        .collect();

    Some(links)
}
