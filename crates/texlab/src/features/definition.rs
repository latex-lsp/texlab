use base_db::Workspace;
use definition::DefinitionParams;
use lsp_types::{GotoDefinitionResponse, LocationLink, Position, Url};

use crate::util::line_index_ext::LineIndexExt;

pub fn goto_definition(
    workspace: &Workspace,
    uri: &Url,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    let document = workspace.lookup(uri)?;
    let offset = document.line_index.offset_lsp(position);
    let params = DefinitionParams {
        workspace,
        document,
        offset,
    };

    let mut links = Vec::new();
    for result in definition::goto_definition(params) {
        let origin_selection_range = Some(
            document
                .line_index
                .line_col_lsp_range(result.origin_selection_range),
        );

        let target_line_index = &result.target.line_index;
        let target_uri = result.target.uri.clone();
        let target_range = target_line_index.line_col_lsp_range(result.target_range);

        let target_selection_range =
            target_line_index.line_col_lsp_range(result.target_selection_range);

        links.push(LocationLink {
            origin_selection_range,
            target_uri,
            target_range,
            target_selection_range,
        });
    }

    Some(GotoDefinitionResponse::Link(links))
}
