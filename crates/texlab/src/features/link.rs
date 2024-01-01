use base_db::{FeatureParams, Workspace};
use lsp_types::{DocumentLink, Url};

use crate::util::to_proto;

pub fn find_all(workspace: &Workspace, uri: &Url) -> Option<Vec<DocumentLink>> {
    let document = workspace.lookup(uri)?;
    let links = links::find_links(FeatureParams::new(workspace, document))
        .into_iter()
        .filter_map(|link| to_proto::document_link(link, &document.line_index))
        .collect();

    Some(links)
}
