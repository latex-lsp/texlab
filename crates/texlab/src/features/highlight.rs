mod label;

use base_db::Workspace;
use lsp_types::DocumentHighlight;

use crate::util::line_index_ext::LineIndexExt;

pub fn find_all(
    workspace: &Workspace,
    params: &lsp_types::DocumentHighlightParams,
) -> Option<Vec<DocumentHighlight>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let document = workspace.lookup(uri)?;
    let position = params.text_document_position_params.position;
    let offset = document.line_index.offset_lsp(position)?;
    label::find_highlights(document, offset)
}
