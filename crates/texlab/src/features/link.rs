use base_db::{FeatureParams, Workspace};
use lsp_types::{DocumentLink, Url};

use crate::util::line_index_ext::LineIndexExt;

pub fn find_all(workspace: &Workspace, uri: &Url) -> Option<Vec<DocumentLink>> {
    let document = workspace.lookup(uri)?;

    let links = links::find_links(FeatureParams::new(workspace, document)).into_iter();
    let links = links.filter_map(|link| {
        Some(lsp_types::DocumentLink {
            data: None,
            tooltip: None,
            target: Some(link.document.uri.clone()),
            range: document.line_index.line_col_lsp_range(link.range)?,
        })
    });

    Some(links.collect())
}
