use base_db::Workspace;

use crate::util::{from_proto, to_proto};

pub fn find(workspace: &Workspace, params: lsp_types::HoverParams) -> Option<lsp_types::Hover> {
    let params = from_proto::hover_params(workspace, params)?;
    let hover = ::hover::find(&params)?;
    to_proto::hover(hover, &params.feature.document.line_index)
}
