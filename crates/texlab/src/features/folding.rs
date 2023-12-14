use base_db::Workspace;
use folding::FoldingRangeKind;

use crate::util::{line_index_ext::LineIndexExt, ClientFlags};

pub fn find_all(
    workspace: &Workspace,
    uri: &lsp_types::Url,
    client_flags: &ClientFlags,
) -> Option<Vec<serde_json::Value>> {
    let document = workspace.lookup(uri)?;
    let foldings = folding::find_all(document)
        .into_iter()
        .filter_map(|folding| {
            let range = document.line_index.line_col_lsp_range(folding.range)?;

            let kind = if client_flags.folding_custom_kinds {
                Some(match folding.kind {
                    FoldingRangeKind::Section => "section",
                    FoldingRangeKind::Environment => "environment",
                    FoldingRangeKind::Entry => "entry",
                })
            } else {
                None
            };

            Some(serde_json::json!({
                "startLine": range.start.line,
                "startCharacter": range.start.character,
                "endLine": range.end.line,
                "endCharacter": range.end.character,
                "kind": kind,
            }))
        });

    Some(foldings.collect())
}
