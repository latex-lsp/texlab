use base_db::{FeatureParams, Workspace};
use highlights::{HighlightKind, HighlightParams};

use crate::util::line_index_ext::LineIndexExt;

pub fn find_all(
    workspace: &Workspace,
    params: &lsp_types::DocumentHighlightParams,
) -> Option<Vec<lsp_types::DocumentHighlight>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let document = workspace.lookup(uri)?;
    let position = params.text_document_position_params.position;
    let offset = document.line_index.offset_lsp(position)?;
    let feature = FeatureParams::new(workspace, document);
    let params = HighlightParams { feature, offset };
    let results = highlights::find_all(params);
    let results = results.into_iter().filter_map(|result| {
        let range = document.line_index.line_col_lsp_range(result.range)?;
        let kind = Some(match result.kind {
            HighlightKind::Write => lsp_types::DocumentHighlightKind::WRITE,
            HighlightKind::Read => lsp_types::DocumentHighlightKind::READ,
        });

        Some(lsp_types::DocumentHighlight { range, kind })
    });

    Some(results.collect())
}
