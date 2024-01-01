use base_db::Workspace;

use crate::util::{from_proto, to_proto};

pub fn find_all(
    workspace: &Workspace,
    params: lsp_types::DocumentHighlightParams,
) -> Option<Vec<lsp_types::DocumentHighlight>> {
    let params = from_proto::highlight_params(workspace, params)?;
    let results = highlights::find_all(&params);
    let results = results.into_iter().filter_map(|result| {
        to_proto::document_highlight(result, &params.feature.document.line_index)
    });

    Some(results.collect())
}
