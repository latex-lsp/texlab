mod latex_include;

use crate::concat_feature;
use crate::feature::FeatureRequest;
use crate::link::latex_include::LatexIncludeLinkProvider;
use lsp_types::{DocumentLink, DocumentLinkParams};

pub struct LinkProvider;

impl LinkProvider {
    pub async fn execute(request: &FeatureRequest<DocumentLinkParams>) -> Vec<DocumentLink> {
        concat_feature!(&request, LatexIncludeLinkProvider)
    }
}
