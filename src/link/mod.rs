mod latex_include;

use crate::link::latex_include::LatexIncludeLinkProvider;
use futures_boxed::boxed;
use lsp_types::{DocumentLink, DocumentLinkParams};
use texlab_workspace::*;

pub struct LinkProvider {
    provider: ConcatProvider<DocumentLinkParams, DocumentLink>,
}

impl LinkProvider {
    pub fn new() -> Self {
        Self {
            provider: ConcatProvider::new(vec![Box::new(LatexIncludeLinkProvider)]),
        }
    }
}

impl FeatureProvider for LinkProvider {
    type Params = DocumentLinkParams;
    type Output = Vec<DocumentLink>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<DocumentLinkParams>,
    ) -> Vec<DocumentLink> {
        self.provider.execute(request).await
    }
}
