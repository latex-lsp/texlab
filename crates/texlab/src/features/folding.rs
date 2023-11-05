use base_db::Workspace;
use folding::FoldingRangeKind;
use lsp_types::{ClientCapabilities, Url};

use crate::util::line_index_ext::LineIndexExt;

pub fn find_all(
    workspace: &Workspace,
    uri: &Url,
    capabilities: &ClientCapabilities,
) -> Option<Vec<serde_json::Value>> {
    let custom_kinds = capabilities
        .text_document
        .as_ref()
        .and_then(|cap| cap.folding_range.as_ref())
        .and_then(|cap| cap.folding_range_kind.as_ref())
        .and_then(|cap| cap.value_set.as_ref())
        .is_some();

    let document = workspace.lookup(uri)?;
    let foldings = folding::find_all(document)
        .into_iter()
        .filter_map(|folding| {
            let range = document.line_index.line_col_lsp_range(folding.range)?;

            let kind = if custom_kinds {
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
