use base_db::Workspace;

use crate::util::{from_proto, to_proto};

pub fn find_all(
    workspace: &Workspace,
    params: lsp_types::InlayHintParams,
) -> Option<Vec<lsp_types::InlayHint>> {
    let params = from_proto::inlay_hint_params(workspace, params)?;
    let hints = inlay_hints::find_all(&params)?
        .into_iter()
        .filter_map(|hint| to_proto::inlay_hint(hint, &params.feature.document.line_index))
        .collect();

    Some(hints)
}
